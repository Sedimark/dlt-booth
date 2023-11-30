-- alias is unique, distinct is not strictly necessary 
SELECT alias
FROM connector.assets, connector.identities
WHERE publisher = connector.identities.id AND eth_address = $1;