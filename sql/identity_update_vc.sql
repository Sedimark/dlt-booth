UPDATE connector.identities
SET vcredential = $1 
WHERE eth_address = $2;