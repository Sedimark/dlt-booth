// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::str::FromStr;

use alloy::{consensus::SignableTransaction, network::TxSigner, primitives::{Address, ChainId, PrimitiveSignature, SignatureError, B256}, signers::{Error, Signer, UnsupportedSignerOperation}};
use async_trait::async_trait;
use alloy::signers::Result;
use crypto::keys::bip44::Bip44;
use iota_sdk::client::{api::GetAddressesOptions, secret::{SecretManage, SecretManager}};
use crate::errors::ConnectorError;

trait ToEIP191 : AsRef<[u8]> {
    const HEADER_EIP191: &[u8; 26] = b"\x19Ethereum Signed Message:\n";
    fn to_eip_191_format(&self) -> Vec<u8> {
      let len = self.as_ref().len().to_string();
      let mut payload = Vec::from(Self::HEADER_EIP191);
      payload.extend_from_slice(len.as_bytes());
      payload.extend_from_slice(self.as_ref());
      payload
    }
}

impl ToEIP191 for &[u8] {}

#[derive(Debug, Clone)]
pub struct IotaSigner<'a>{
    secret_manager: &'a SecretManager,
    address: Address,
    address_chain: Bip44,
    chain_id: Option<u64>
}

impl<'a> IotaSigner<'a>{
    /// Generate a new signer instance
    pub async fn new(secret_manager: &'a SecretManager, chain_id: Option<u64>, address_chain: Bip44) -> Result<Self, ConnectorError> {
        let address_index = address_chain.address_index;
        let options = GetAddressesOptions::default()
        .with_range(address_index..address_index+1)
        .with_coin_type(address_chain.coin_type)
        .with_account_index(address_chain.account);

        let address = secret_manager.generate_evm_addresses(options)
            .await?.first()
            .ok_or(ConnectorError::WalletError("Cannot create EVM address".to_owned()))?
            .trim_start_matches("0x")
            .to_owned();

        let address: Address = Address::from_str(&address)?;

        Ok(Self {secret_manager, chain_id, address_chain, address })
    }

    /// Sign using evm address keypair
    async fn sign_evm_data(&self, payload: impl AsRef<[u8]>) -> Result<Vec<u8>, ConnectorError>{
        let signature = self.secret_manager.sign_secp256k1_ecdsa(&payload.as_ref(), self.address_chain).await?.1;
        let mut signature = Vec::from(signature.to_bytes());

        // it is required to add 27 to the last byte in order to pass EVM verification
        // https://bitcoin.stackexchange.com/questions/38351/ecdsa-v-r-s-what-is-v/38909#comment46061_38909
        let last_byte = signature.last_mut().ok_or(ConnectorError::OtherError("cannot change the last byte".to_owned()))?;
        *last_byte += 27;

        log::debug!("signature completed: {:?}", signature);
        Ok(signature)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Signer for IotaSigner<'_> {
    async fn sign_hash(&self, _hash: &B256) -> Result<PrimitiveSignature> {
        Err(Error::UnsupportedOperation(UnsupportedSignerOperation::SignHash))
    }

    #[inline]
    async fn sign_message(&self, message: &[u8]) -> Result<PrimitiveSignature> {
        let eip191_bytes = message.to_eip_191_format();
        let signature = self.sign_evm_data(eip191_bytes)
            .await
            .map_err(|_| {alloy::signers::Error::SignatureError(alloy::primitives::SignatureError::FromBytes(""))})?;
        let primitive = PrimitiveSignature::try_from(signature.as_slice())?;
        Ok(primitive)
    }

    #[cfg(feature = "eip712")]
    #[inline]
    async fn sign_typed_data<T: SolStruct + Send + Sync>(
        &self,
        payload: &T,
        domain: &Eip712Domain,
    ) -> Result<Signature> {
        self.sign_typed_data_(&payload.eip712_hash_struct(), domain)
            .await
            .map_err(alloy_signer::Error::other)
    }

    #[cfg(feature = "eip712")]
    #[inline]
    async fn sign_dynamic_typed_data(&self, payload: &TypedData) -> Result<Signature> {
        self.sign_typed_data_(&payload.hash_struct()?, &payload.domain)
            .await
            .map_err(alloy_signer::Error::other)
    }

    #[inline]
    fn address(&self) -> Address {
        self.address
    }

    #[inline]
    fn chain_id(&self) -> Option<ChainId> {
        self.chain_id
    }

    #[inline]
    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.chain_id = chain_id;
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl TxSigner<PrimitiveSignature> for IotaSigner<'_>{
    /// Get the address of the signer.
    fn address(&self) -> Address{
        self.address
    }

    /// Asynchronously sign an unsigned transaction.
    #[doc(alias = "sign_tx")]
    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<PrimitiveSignature>,
    ) -> alloy::signers::Result<PrimitiveSignature>{
        let rlp = tx.encoded_for_signing();
        let signature = self.sign_evm_data(rlp).await
            .map_err(|_| {Error::Other("stronghold cannot perform the signature".into())})?;
        PrimitiveSignature::try_from(signature.as_slice())
            .map_err(|_| {Error::SignatureError(SignatureError::FromBytes("cannot convert from stronghold signature"))})
    }
}
#[cfg(test)]
mod tests{
    use alloy::{hex::ToHexExt, signers::Signer};
    use crypto::keys::{bip39::Mnemonic, bip44::Bip44};
    use iota_sdk::client::secret::SecretManager;

    use crate::utils::alloy_signer::{IotaSigner, ToEIP191};


    #[tokio::test]
    async fn eip191_format(){
      let message = b"\x19Ethereum Signed Message:\n5nonce";
      assert_eq!("nonce".as_bytes().to_eip_191_format(), message)
    }

    #[tokio::test]
    async fn sign_evm_test(){
      let secret_manager = SecretManager::try_from_mnemonic(
        Mnemonic::from("grace eye hour away retire put crunch burger bracket coyote twist cherry glare collect retreat")
        ).unwrap();
      
      //let iota = crate::utils::iota::IotaState::init(key_storage, wallet_storage, dlt_config, evm_config).await.unwrap();
      let address = Bip44::new(60).with_account(0).with_address_index(0);
      let signer = IotaSigner::new(&secret_manager, Some(1773), address).await.unwrap();

      let signature =signer.sign_message(b"nonce")
        .await;

      assert!(signature.is_ok(), "Signature error: {:?}", signature.err().unwrap().to_string());

      let signature = signature.unwrap().as_bytes().encode_hex_with_prefix();
  
      assert_eq!(signature, "0x2ef9407839892b05046a9ee7e3e37632c6ff644d198bd64a20371be08d051c680ad25bfecb4ca783835ce3ba087633a5816382ff131085b5ea34b8c03a25f0c41b")
    }
}