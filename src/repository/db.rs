use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;

use crate::models::client::Client;
use crate::models::last_transaction::LastTransaction;
use crate::models::schema::clients::dsl::*;
use crate::models::schema::last_transactions::dsl::*;
use crate::models::schema::transactions::dsl::*;
use crate::models::transaction::{NewTransaction, Transaction};

pub type DBPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pub pool: DBPool,
}

impl Database {
    pub fn new() -> Self {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let result = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        Database { pool: result }
    }

    pub fn get_clients(&self) -> Vec<Client> {
        clients
            .load::<Client>(&mut self.pool.get().unwrap())
            .expect("Failed to get clients.")
    }

    pub fn get_client(&self, find_id: i32) -> Option<Client> {
        clients
            .find(find_id)
            .first::<Client>(&mut self.pool.get().unwrap())
            .ok()
    }

    pub fn update_client(&self, client: Client) -> Result<Client, diesel::result::Error> {
        diesel::update(clients.filter(crate::models::schema::clients::dsl::id.eq(client.id)))
            .set(&client)
            .get_result(&mut self.pool.get().unwrap())
    }

    pub fn create_transaction(
        &self,
        transaction: NewTransaction,
    ) -> Result<Transaction, diesel::result::Error> {
        diesel::insert_into(transactions)
            .values(&transaction)
            .get_result(&mut self.pool.get().unwrap())
    }

    pub fn get_last_transactions(&self, find_id: i32) -> Vec<LastTransaction> {
        last_transactions
            .filter(crate::models::schema::last_transactions::dsl::client_id.eq(find_id))
            .select(LastTransaction::as_select())
            .load::<LastTransaction>(&mut self.pool.get().unwrap())
            .expect("Error loading transactions")
    }
}
