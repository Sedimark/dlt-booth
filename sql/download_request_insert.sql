INSERT INTO connector.download_requests(nonce, requester_did, expiration)
VALUES ($1, $2, $3)
RETURNING $table_fields;