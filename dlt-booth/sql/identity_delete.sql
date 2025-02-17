-- SPDX-FileCopyrightText: 2024 Fondazione LINKS
-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

DELETE FROM dlt_booth.identities
WHERE did = $1;