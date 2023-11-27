INSERT INTO connector.identities(eth_address, did, fragment)
VALUES ($1, $2, $3)
RETURNING $table_fields;