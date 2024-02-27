-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

INSERT INTO connector.identities(eth_address, did, fragment)
VALUES ($1, $2, $3)
RETURNING $table_fields;