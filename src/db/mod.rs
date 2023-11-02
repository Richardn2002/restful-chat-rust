pub mod auth;

use auth::Auth;

use std::error::Error;

use mongodb::Client;

/// representation of the entire MongoDb database
#[derive(Clone)]
pub struct Db {
    /// handle of the connection to database
    conn: Client,
    /// handle of auth database
    /// option as not all server nodes need to access auth
    auth: Option<Auth>,
}

impl Db {
    pub async fn new(uri: String) -> Result<Db, Box<dyn Error>> {
        let conn = Client::with_uri_str(uri).await?;

        Ok(Db { conn, auth: None })
    }
}
