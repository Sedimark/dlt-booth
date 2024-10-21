-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

DELETE
FROM connector.download_requests 
WHERE expiration < $1;