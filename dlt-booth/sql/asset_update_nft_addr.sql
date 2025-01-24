-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

UPDATE dlt_booth.assets
SET nft_address = $1
WHERE alias = $2
RETURNING $table_fields;