-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

SELECT $table_fields 
FROM dlt_booth.download_requests 
WHERE requester_did = $1 
AND nonce = $2
AND expiration >= $3;