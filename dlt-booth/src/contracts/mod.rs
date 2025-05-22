// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use alloy::{providers::{fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller}, Identity, RootProvider}, sol};

pub type ScProvider = FillProvider<JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>, RootProvider>;
sol!(
    #[sol(rpc)]
    ServiceBase,
    "../smart-contracts/ServiceBase.json");

sol!(
    #[sol(rpc)]
    Factory,
    "../smart-contracts/Factory.json"
);