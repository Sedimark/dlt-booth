// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::{body::MessageBody, dev::{ServiceRequest, ServiceResponse}, Error, HttpMessage};
use actix_web_lab::middleware::Next;
use std::collections::HashMap;

use actix_web::web;
use deadpool_postgres::Pool;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::{core::{Object, Timestamp}, credential::{DecodedJwtCredential, DecodedJwtPresentation, FailFast, Jwt, JwtCredentialValidationOptions, JwtCredentialValidator, JwtCredentialValidatorUtils, JwtPresentationValidationOptions, JwtPresentationValidator, JwtPresentationValidatorUtils, SubjectHolderRelationship}, did::{CoreDID, DID}, document::verifiable::JwsVerificationOptions, iota::IotaDocument, resolver::Resolver, verification::{jws::JwsHeader, jwu::decode_b64_json}};
use crate::{dtos::ProofOfPurchaseRequest, errors::ConnectorError, repository::download_request_operations::ChallengesExt, utils::iota::IotaState};

// TODO: handle expiration and errors (no unwraps)
pub async fn verify_presentation_jwt(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    log::info!("Hi from start 1. You requested: {}", req.path());
    let db_pool = req.app_data::<web::Data<Pool>>().ok_or(ConnectorError::MiddlewareError("no db pool".to_string())).unwrap();
    let iota_state = req.app_data::<web::Data<IotaState>>().ok_or(ConnectorError::MiddlewareError("no iota state".to_string())).unwrap();

    // Extract the JWT from the request.
    let bearer = req.headers().get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_whitespace().nth(1));
    
    if let Some(jwt_str) = bearer {
        log::info!("Presentation jwt: {}", jwt_str);

        let presentation_jwt = Jwt::from(jwt_str.to_string());

        // ===========================================================================
        // Verifier receives the Verifiable Presentation and verifies it.
        // ===========================================================================

        // The verifier wants the following requirements to be satisfied:
        // - JWT verification of the presentation (including checking the requested challenge to mitigate replay attacks)
        // - JWT verification of the credentials.
        // - The presentation holder must always be the subject, regardless of the presence of the nonTransferable property
        // - The issuance date must not be in the future.

        // retrieve the header
        let header_b64 = presentation_jwt.as_str().split('.').next().unwrap_or("");
        let header = decode_b64_json::<JwsHeader>(header_b64)
            .map_err(|e| {ConnectorError::JwtValidationError(identity_iota::credential::JwtValidationError::JwsDecodingError(e))})?;
        
        let nonce = header.nonce().ok_or(ConnectorError::ChallengeMissing)?;

        let mut resolver: Resolver<IotaDocument> = Resolver::new();
        resolver.attach_iota_handler(iota_state.wallet.client().clone());

        // Resolve the holder's document.
        let holder_did: CoreDID = JwtPresentationValidatorUtils::extract_holder(&presentation_jwt).unwrap();
        let holder = resolver.resolve(&holder_did).await.unwrap();

        // Recover the expected challenge from the database
        let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
        log::info!("Holder did: {}", holder.id());
        // check and clean holder requests // TODO: clean? the clean currently is done each time the GET api is called
        let download_request = pg_client.get_challenge(&holder.id().to_string(), &nonce.to_string(), Timestamp::now_utc()).await?;

        let presentation_verifier_options = JwsVerificationOptions::default().nonce(download_request.nonce.clone());

        // Validate presentation. Note that this doesn't validate the included credentials.
        let presentation_validation_options = JwtPresentationValidationOptions::default().presentation_verifier_options(presentation_verifier_options);
        let presentation: DecodedJwtPresentation<Jwt> = JwtPresentationValidator::with_signature_verifier(
            EdDSAJwsVerifier::default(),
        )
        .validate(&presentation_jwt, &holder, &presentation_validation_options).unwrap();

        // Concurrently resolve the issuers' documents.
        let jwt_credentials: &Vec<Jwt> = &presentation.presentation.verifiable_credential;
        let issuers: Vec<CoreDID> = jwt_credentials
            .iter()
            .map(JwtCredentialValidatorUtils::extract_issuer_from_jwt)
            .collect::<Result<Vec<CoreDID>, _>>().unwrap();
        let issuers_documents: HashMap<CoreDID, IotaDocument> = resolver.resolve_multiple(&issuers).await.unwrap();

        // Validate the credentials in the presentation.
        let credential_validator: JwtCredentialValidator<EdDSAJwsVerifier> =
            JwtCredentialValidator::with_signature_verifier(EdDSAJwsVerifier::default());
        let validation_options: JwtCredentialValidationOptions = JwtCredentialValidationOptions::default()
            .subject_holder_relationship(holder_did.to_url().into(), SubjectHolderRelationship::AlwaysSubject);

        // TODO: verify that the credential is not revoked by iteracting with the Identity SC

        for (index, jwt_vc) in jwt_credentials.iter().enumerate() {
            // SAFETY: Indexing should be fine since we extracted the DID from each credential and resolved it.
            let issuer_document: &IotaDocument = &issuers_documents[&issuers[index]];

            let _decoded_credential: DecodedJwtCredential<Object> = credential_validator
            .validate::<_, Object>(jwt_vc, issuer_document, &validation_options, FailFast::FirstError)
            .unwrap();
        }
        // Since no errors were thrown by `verify_presentation` we know that the validation was successful.
        println!("VP successfully validated: {:#?}", presentation.presentation);

        // ===========================================================================
        // Verifier extracts information for verifcation of the proof of purchase
        // ===========================================================================

        // Extract the verifcation method from the holder document
        let vm = holder.resolve_method("#ethAddress", None).ok_or(ConnectorError::EthMethodNotFound)?;

        vm.type_().to_string().eq("EcdsaSecp256k1RecoverySignature2020").then(|| Some(())).ok_or(ConnectorError::InvalidVerificationMethodType)?;

        // Extract the ethereum address from the verifcation method
        let eth_addr = vm.data().custom()
        .take_if(|method_data| {method_data.name == "blockchainAccountId"} )
        .ok_or(ConnectorError::InvalidVerificationMethodType)?
        .data.as_str().ok_or(ConnectorError::InvalidVerificationMethodType)?
        .strip_prefix("eip155:1:")
        .ok_or(ConnectorError::InvalidVerificationMethodType)?;

        // Extract wallet signature from presentation custom claims
        let custom_claims: std::collections::BTreeMap<String, serde_json::Value> = presentation.custom_claims
        .ok_or::<ConnectorError>(ConnectorError::MiddlewareError("custom_claims not found".to_string()).into())?;

        let eth_signature = custom_claims.get("walletSignature")
        .ok_or::<ConnectorError>(ConnectorError::MiddlewareError("walletSignature not found".to_string()).into())?
        .as_str()
        .ok_or::<ConnectorError>(ConnectorError::MiddlewareError("couldn't convert Value to str".to_string()).into())?;

        // Extract wallet signature from the presentation 
        log::info!("{}", eth_signature); // TODO verify the signature? 

        // TODO: move in ver_proof_of_purchase middleware, just pass the DecodedJwtPresentation and the holder document
        req.extensions_mut()
        .insert( ProofOfPurchaseRequest {
            nonce: download_request.nonce.clone(),
            eth_address: eth_addr.to_string(),
            eth_signature: eth_signature.to_string(),
            did: holder.id().to_string().clone(),
        });

        next.call(req).await
    // post-processing
    } else {
        // If authorization header is not present or malformed, return an error response
        return Err(ConnectorError::MiddlewareError("no jwt".to_string()).into());
    }
    
}

