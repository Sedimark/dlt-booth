use identity_iota::verification::jwu::decode_b64_json;
use serde_json::Value;

/// Decode a verifiable credential JWT without the Issuer DID document. 
/// 
/// Use it only if the credential has been already verified.
pub fn decode_vc_unverified(jwt: &str) -> Option<Value>{

    jwt
        .split('.')
        .skip(1)
        .next()
        .and_then(|str| decode_b64_json::<Value>(str).ok())
}