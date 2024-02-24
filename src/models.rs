use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};

#[derive(Serialize, FromRow)]
pub struct Client {
    pub id: i32,
    pub account_limit: i64,
    pub balance: i64,
}

#[derive(Deserialize)]
pub struct TransactionPayload {
    pub valor: i64,
    pub tipo: String,
    pub descricao: String,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    pub limite: i64,
    pub saldo: i64,
}

#[derive(Serialize)]
pub struct GetLastTransactionsResponse {
    pub saldo: Balance,
    pub ultimas_transacoes: Vec<LastTransaction>,
}

#[derive(Serialize)]
pub struct Balance {
    pub total: i64,
    pub data_extrato: String,
    pub limite: i64,
}

#[derive(Serialize, FromRow)]
pub struct LastTransaction {
    pub valor: i64,
    pub tipo: String,
    pub descricao: String,
    pub realizada_em: String,
}
