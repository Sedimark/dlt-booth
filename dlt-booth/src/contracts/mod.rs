// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use alloy::sol;

sol!(
    #[sol(rpc)]
    ServiceBase,
    "../smart-contracts/ServiceBase.json");

sol!(
    #[sol(rpc)]
    Factory,
    "../smart-contracts/Factory.json"
);

sol!(
    #[sol(rpc)]
    AccessTokenBase,
    "../smart-contracts/AccessTokenBase.json"
);

sol!(
    #[sol(rpc)]
    FixedRateExchange,
    "../smart-contracts/FixedRateExchange.json"
);
