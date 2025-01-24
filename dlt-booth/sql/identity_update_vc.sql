-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

UPDATE dlt_booth.identities
SET vcredential = $1 
WHERE eth_address = $2
RETURNING $table_fields;