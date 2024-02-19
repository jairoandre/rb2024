use crate::AppState;
use actix_web::{
    get, post,
    web::{scope, Data, Json, Path, ServiceConfig},
    HttpResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};
use std::time::SystemTime;

fn iso8601(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

#[derive(Serialize, FromRow)]
pub struct Client {
    pub id: i32,
    pub account_limit: i32,
    pub balance: i32,
}


#[get("")]
async fn get_clients(state: Data<AppState>) -> HttpResponse {
    match sqlx::query_as::<_, Client>("SELECT * FROM clients")
        .fetch_all(&state.db)
        .await
    {
        Ok(clients) => HttpResponse::Ok().json(clients),
        Err(e) => {
            println!("{}", e);
            HttpResponse::NotFound().json("No users found")
        }
    }
}

#[get("/{id}")]
async fn get_client(state: Data<AppState>, path: Path<i32>) -> HttpResponse {
    let id = path.into_inner();
    match sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(client) => HttpResponse::Ok().json(client),
        Err(_) => HttpResponse::NotFound().json("No user found"),
    }
}

#[derive(Deserialize)]
pub struct TransactionPayload {
    valor: i32,
    tipo: String,
    descricao: String,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    limite: i32,
    saldo: i32,
}

#[post("/{id}/transacoes")]
async fn post_transaction(
    state: Data<AppState>,
    path: Path<i32>,
    payload: Json<TransactionPayload>,
) -> HttpResponse {
    let client_id = path.into_inner();
    match sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
        .bind(client_id)
        .fetch_one(&state.db)
        .await
    {
        Ok(mut client) => {
            match payload.tipo.as_str() {
                "c" => {
                    client.balance += payload.valor;
                }
                "d" => {
                    client.balance -= payload.valor;
                }
                _ => {
                    return HttpResponse::BadRequest().body("Invalid operation");
                }
            }
            if (client.balance + client.account_limit) < 0 {
                return HttpResponse::UnprocessableEntity().body("Account limit reached!");
            }
            match sqlx::query("UPDATE clients SET balance = $1 WHERE id = $2 RETURNING *")
                .bind(client.balance)
                .bind(client.id)
                .execute(&state.db)
                .await {
                    Ok(_) => {
                        match sqlx::query("INSERT INTO transactions (client_id, amount, transaction_type, details) VALUES ($1, $2, $3, $4)")
                        .bind(client.id)
                        .bind(payload.valor)
                        .bind(payload.tipo.clone())
                        .bind(payload.descricao.clone())
                        .execute(&state.db)
                        .await {
                            Ok(_) =>  HttpResponse::Ok().json(TransactionResponse {
                                limite: client.account_limit,
                                saldo: client.balance
                            }),
                            Err(_) => HttpResponse::InternalServerError().body("Internal server error")

                        }
                }
                Err(_) => HttpResponse::InternalServerError().body("Error updating client."),
            }
        }
        Err(_) => {
            return HttpResponse::NotFound().body("Client not found");
        }
    }
}

#[derive(Serialize, FromRow)]
struct LastTransaction {
    valor: i32,
    tipo: String,
    descricao: String,
    realizada_em: String,
}

#[derive(Serialize)]
struct GetLastTransactionsResponse {
    saldo: Balance,
    ultimas_transacoes: Vec<LastTransaction>,
}

#[derive(Serialize)]
struct Balance {
    total: i32,
    data_extrato: String,
    limite: i32,
}

static LAST_TRANSACTION_SQL: &str = "
select 
lt.amount as valor, 
lt.transaction_type as tipo, 
lt.details as descricao, 
to_char(lt.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MSZ') as realizada_em
from last_transactions lt where lt.client_id = $1;
";

#[get("/{id}/extrato")]
async fn get_extract(state: Data<AppState>, path: Path<i32>) -> HttpResponse {
    let client_id = path.into_inner();
    match sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
        .bind(client_id)
        .fetch_one(&state.db)
        .await
    {
        Ok(client) => {
            match sqlx::query_as::<_, LastTransaction>(LAST_TRANSACTION_SQL)
                .bind(client_id)
                .fetch_all(&state.db)
                .await
            {
                Ok(last_transactions) => HttpResponse::Ok().json(GetLastTransactionsResponse {
                    saldo: Balance {
                        total: client.balance,
                        data_extrato: iso8601(&SystemTime::now()),
                        limite: client.account_limit,
                    },
                    ultimas_transacoes: last_transactions,
                }),
                Err(_) => {
                    HttpResponse::InternalServerError().body("Could not load last transactions")
                }
            }
        }
        Err(_) => HttpResponse::NotFound().body("Client not found"),
    }
}

pub fn init_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/clientes")
            .service(get_clients)
            .service(get_client)
            .service(post_transaction)
            .service(get_extract),
    );
}
