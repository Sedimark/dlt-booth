// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use ethers::contract::abigen;
use alloy::sol;

abigen!(ServiceBase, "../smart-contracts/ServiceBase.json");