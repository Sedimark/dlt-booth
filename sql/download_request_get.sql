SELECT $table_fields 
FROM connector.download_requests 
WHERE requester_did = $1;