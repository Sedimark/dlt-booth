-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

INSERT INTO dlt_booth.download_requests(nonce, requester_did, expiration)
VALUES ($1, $2, $3)
RETURNING $table_fields;