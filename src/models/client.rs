use diesel::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::clients)]
pub struct Client {
    pub id: i32,
    pub account_limit: i32,
    pub balance: i32,
}
