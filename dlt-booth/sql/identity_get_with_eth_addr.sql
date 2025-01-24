-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

SELECT $table_fields FROM dlt_booth.identities WHERE eth_address = $1;