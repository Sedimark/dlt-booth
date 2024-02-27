-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

SELECT $table_fields 
FROM connector.download_requests 
WHERE requester_did = $1;