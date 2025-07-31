-- SPDX-FileCopyrightText: 2025 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

SELECT evm_address
FROM dlt_booth.addresses
WHERE addr_name = $1;