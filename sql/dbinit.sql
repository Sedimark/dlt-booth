CREATE SCHEMA IF NOT EXISTS connector;

CREATE TABLE connector.identities (
	id          BIGSERIAL PRIMARY KEY,
	eth_address TEXT NOT NULL,
	did         TEXT NOT NULL,
	fragment    TEXT NOT NULL,
	vcredential TEXT,
	UNIQUE (eth_address)
);

CREATE TABLE connector.assets (
	id 				BIGSERIAL PRIMARY KEY,
    nft_address 	TEXT,
    cid 			TEXT NOT NULL,
    alias 			TEXT NOT NULL,
    asset_path 		TEXT NOT NULL,
    offering_path 	TEXT NOT NULL,
    asset_hash 		TEXT NOT NULL,
    offering_hash 	TEXT NOT NULL,
    sign 			TEXT NOT NULL,
	publisher 		BIGINT NOT NULL REFERENCES connector.identities(id) ON DELETE RESTRICT,
	UNIQUE (nft_address, cid, alias)
);

CREATE TABLE connector.download_requests (
  	nonce 				UUID PRIMARY KEY,
  	asset_id 			BIGINT NOT NULL REFERENCES connector.assets(id) ON DELETE RESTRICT,
  	requester_did 		TEXT NOT NULL,
  	-- expiration		TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
  	expiration			TEXT NOT NULL
);