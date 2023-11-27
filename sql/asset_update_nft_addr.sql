UPDATE connector.assets
SET nft_address = $1
WHERE alias = $2;