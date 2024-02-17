use diesel::{Selectable, Queryable};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name=crate::models::schema::last_transactions)]
pub struct LastTransaction {
    pub id: i32,
    pub client_id: i32,
    pub amount: i32,
    pub transaction_type: String,
    pub details: String,
    pub created_at: SystemTime,
    pub balance: i32,
    pub account_limit: i32,
}
