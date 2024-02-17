use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name=crate::models::schema::transactions)]
pub struct Transaction {
    pub id: i32,
    pub client_id: i32,
    pub amount: i32,
    pub transaction_type: String,
    pub details: String,
    pub created_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name=crate::models::schema::transactions)]
pub struct NewTransaction {
    pub client_id: i32,
    pub amount: i32,
    pub transaction_type: String,
    pub details: String,
}
