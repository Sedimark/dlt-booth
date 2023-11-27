use deadpool_postgres::Client as PostgresClient;
use crate::{repository::identity_operations::IdentityExt, models::identity::Identity, errors::ConnectorError};

pub async fn create_identity(
    pg_client: &PostgresClient,
    eth_address: String, 
) -> Result<String, ConnectorError>  {
    log::info!("creating identity");
    let new_identity = Identity {
        eth_address,
        did: "todo!()".to_string(),
        fragment: "todo!()".to_string(),
        vcredential: None,
    };
    let _ = pg_client.insert_identity(&new_identity).await?;
    
    Ok("hello".to_string())
}