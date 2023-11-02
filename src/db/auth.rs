use super::Db;
use crate::types::UserId;

use bson::doc;
use mongodb::{
    options::{FindOneOptions, Hint},
    Collection, IndexModel,
};
use serde::{Deserialize, Serialize};

/// scheme for (username, password, user id) credentials
#[derive(Serialize, Deserialize)]
struct Cred {
    /// unique string specified by user during registration
    uname: String,
    /// bcrypt-ed password
    pwd: String,
    /// unique user id used for indexing content
    uid: UserId,
}

/// representation of the auth database
#[derive(Clone)]
pub struct Auth {
    /// handle to the credentials collection
    creds: Collection<Cred>,
}

impl Db {
    pub async fn init_auth(
        mut self,
        db_name: String,
        creds_name: String,
    ) -> Result<Db, Box<dyn std::error::Error>> {
        let auth_db = self.conn.database(&db_name);

        let existing_colls = auth_db.list_collection_names(None).await?;

        let creds_coll = auth_db.collection(&creds_name);
        if !existing_colls.contains(&creds_name) {
            println!("Collection {creds_name} in Database {db_name} non-existent. Creating.");

            auth_db.create_collection(&creds_name, None).await?;

            let keys = doc! {
                "uname": "hashed"
            };
            let model = IndexModel::builder().keys(keys).build();
            creds_coll.create_index(model, None).await?;
        }

        self.auth = Some(Auth { creds: creds_coll });
        Ok(self)
    }
}

impl Db {
    /// return (hashed password, user id) pair for given username
    pub async fn query_creds(
        &self,
        uname: &str,
    ) -> Result<Option<(String, UserId)>, mongodb::error::Error> {
        let filter = doc! {
            "uname": doc! {
                "$eq": uname.to_owned()
            }
        };
        let opts = FindOneOptions::builder()
            .hint(Hint::Name("uname_\"hashed\"".to_string()))
            .build();
        let res = self
            .auth
            .as_ref()
            .unwrap()
            .creds
            .find_one(filter, opts)
            .await?;

        if let Some(entry) = res {
            Ok(Some((entry.pwd, entry.uid)))
        } else {
            Ok(None)
        }
    }

    /// insert (hashed password, user id) pair for given username
    pub async fn insert_creds(
        &self,
        uname: &str,
        pwd: &str,
        uid: UserId,
    ) -> Result<(), mongodb::error::Error> {
        let doc = Cred {
            uname: uname.to_owned(),
            pwd: pwd.to_owned(),
            uid,
        };

        self.auth
            .as_ref()
            .unwrap()
            .creds
            .insert_one(doc, None)
            .await?;

        Ok(())
    }
}
