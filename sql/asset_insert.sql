INSERT INTO connector.assets(nft_address, cid, alias, asset_path, offering_path, asset_hash, offering_hash, sign, publisher)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
RETURNING $table_fields;