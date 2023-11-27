INSERT INTO connector.download_requests(nonce, asset_id, requester_did, expiration)
VALUES ($1, $2, $3, $4)
RETURNING $table_fields;