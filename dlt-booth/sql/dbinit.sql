-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

CREATE SCHEMA IF NOT EXISTS dlt_booth;

CREATE TABLE dlt_booth.identities (
	id          BIGSERIAL PRIMARY KEY,
	eth_address TEXT NOT NULL,
	did         TEXT NOT NULL,
	fragment    TEXT NOT NULL,
	vcredential TEXT,
	UNIQUE (eth_address)
);

CREATE TABLE dlt_booth.assets (
	id 				BIGSERIAL PRIMARY KEY,
    nft_address 	TEXT,
    cid 			TEXT NOT NULL,
    alias 			TEXT NOT NULL,
    asset_path 		TEXT NOT NULL,
    offering_path 	TEXT NOT NULL,
    asset_hash 		TEXT NOT NULL,
    offering_hash 	TEXT NOT NULL,
    sign 			TEXT NOT NULL,
	publisher 		BIGINT NOT NULL REFERENCES dlt_booth.identities(id) ON DELETE RESTRICT,
	UNIQUE (nft_address, cid, alias)
);

CREATE TABLE dlt_booth.download_requests (
  	nonce 				TEXT PRIMARY KEY,
  	-- asset_id 		BIGINT NOT NULL REFERENCES dlt_booth.assets(id) ON DELETE RESTRICT,
  	requester_did 		TEXT NOT NULL,
  	-- expiration		TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
  	expiration			TEXT NOT NULL
);