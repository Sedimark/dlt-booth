// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use alloy::{network::Ethereum, providers::{fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller}, Identity, RootProvider}, sol, transports::http::{Client, Http}};

pub type ScProvider = FillProvider<JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>, RootProvider<Http<Client>>, Http<Client>, Ethereum>;

sol!(
    #[sol(rpc)]
    ServiceBase,
    "../smart-contracts/ServiceBase.json");

sol!(
    #[sol(rpc)]
    Factory, 
    "../smart-contracts/Factory.json");