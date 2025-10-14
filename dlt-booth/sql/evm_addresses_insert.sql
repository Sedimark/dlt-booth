-- SPDX-FileCopyrightText: 2025 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

INSERT INTO dlt_booth.addresses(addr_name, evm_address)
VALUES ($1, $2)
ON CONFLICT (addr_name) DO UPDATE 
SET evm_address = $2;