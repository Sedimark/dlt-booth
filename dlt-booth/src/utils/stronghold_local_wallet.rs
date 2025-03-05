// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use alloy::{consensus::{SignableTransaction, TxEnvelope, TypedTransaction}, network::{AnyNetwork, Network, NetworkWallet, TxSigner}, primitives::{Address, PrimitiveSignature}};
use super::alloy_signer::IotaSigner;

#[derive(Debug, Clone)]
pub struct StrongholdWallet<'a>(IotaSigner<'a>);

impl<'a> StrongholdWallet<'a>{
    pub fn new(signer: IotaSigner<'a>) -> Self
    {
        Self(signer)
    }

    #[doc(alias = "sign_tx_inner")]
    async fn sign_transaction_inner(
        &self,
        tx: &mut dyn SignableTransaction<PrimitiveSignature>,
    ) -> alloy::signers::Result<PrimitiveSignature> {
        self.0
            .sign_transaction(tx)
            .await
    }
}


impl<'a,N> NetworkWallet<N> for StrongholdWallet<'a>
where
    N: Network<UnsignedTx = TypedTransaction, TxEnvelope = TxEnvelope>,
{
    fn default_signer_address(&self) -> Address {
        TxSigner::address(&self.0)
    }

    fn has_signer_for(&self, address: &Address) -> bool {
        TxSigner::address(&self.0).eq(address)
    }

    fn signer_addresses(&self) -> impl Iterator<Item = Address> {
        [TxSigner::address(&self.0)].into_iter()
    }

    #[doc(alias = "sign_tx_from")]
    async fn sign_transaction_from(
        &self,
        sender: Address,
        tx: TypedTransaction,
    ) -> alloy::signers::Result<TxEnvelope> {
        let _ = sender;
        match tx {
            TypedTransaction::Legacy(mut t) => {
                let sig = self.sign_transaction_inner(&mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            TypedTransaction::Eip2930(mut t) => {
                let sig = self.sign_transaction_inner(&mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            TypedTransaction::Eip1559(mut t) => {
                let sig = self.sign_transaction_inner(&mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            TypedTransaction::Eip4844(mut t) => {
                let sig = self.sign_transaction_inner(&mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            TypedTransaction::Eip7702(mut t) => {
                let sig = self.sign_transaction_inner(&mut t).await?;
                Ok(t.into_signed(sig).into())
            }
        }
    }
}

/*#[cfg(test)]
mod tests{
    use alloy::{hex::ToHexExt, signers::Signer};
    use crypto::keys::{bip39::Mnemonic, bip44::Bip44};
    use iota_sdk::client::secret::SecretManager;

    use crate::utils::alloy_signer::{IotaSigner};


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
}*/