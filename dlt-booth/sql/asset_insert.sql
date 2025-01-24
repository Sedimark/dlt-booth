-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

INSERT INTO dlt_booth.assets(cid, alias, asset_path, offering_path, asset_hash, offering_hash, sign, publisher)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
RETURNING $table_fields;