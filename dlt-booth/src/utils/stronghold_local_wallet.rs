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