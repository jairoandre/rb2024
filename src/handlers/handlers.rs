use crate::{
    models::transaction::{NewTransaction},
    repository::db::Database,
};
use actix_web::{get, post, web, HttpResponse};
use chrono::prelude::{DateTime, Utc};
use log;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

fn iso8601(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

#[get("")]
async fn get_clients(db: web::Data<Database>) -> HttpResponse {
    let clients = db.get_clients();
    HttpResponse::Ok().json(clients)
}

#[get("/{id}")]
async fn get_client(db: web::Data<Database>, path: web::Path<i32>) -> HttpResponse {
    let client = db.get_client(path.into_inner());
    match client {
        Some(client) => HttpResponse::Ok().json(client),
        None => HttpResponse::NotFound().body("Not Found"),
    }
}

#[derive(Deserialize)]
struct TransactionPayload {
    valor: i32,
    tipo: String,
    descricao: String,
}

#[derive(Serialize, Deserialize)]
struct CreateTransactionResponse {
    limite: i32,
    saldo: i32,
}

#[post("/{id}/transacoes")]
async fn post_transaction(
    db: web::Data<Database>,
    client_id: web::Path<i32>,
    payload: web::Json<TransactionPayload>,
) -> HttpResponse {
    let client = db.get_client(client_id.into_inner());
    match client {
        Some(mut client) => {
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
            let updated_client = db.update_client(client);
            match updated_client {
                Ok(client) => {
                    log::info!("Client updated.");
                    let new_transaction = NewTransaction {
                        client_id: client.id,
                        amount: payload.valor,
                        transaction_type: payload.tipo.clone(),
                        details: payload.descricao.clone(),
                    };
                    let _transaction = db.create_transaction(new_transaction);
                    let create_response = CreateTransactionResponse {
                        limite: client.account_limit,
                        saldo: client.balance,
                    };
                    HttpResponse::Ok().json(create_response)
                }
                Err(_e) => HttpResponse::InternalServerError().body("Error updating client."),
            }
        }
        None => {
            return HttpResponse::NotFound().body("Client not found");
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Extract {
    saldo: Option<Balance>,
    ultimas_transacoes: Option<Vec<LastTransaction>>,
}

#[derive(Serialize, Deserialize)]
struct Balance {
    total: i32,
    data_extrato: String,
    limite: i32,
}

#[derive(Serialize, Deserialize)]
struct LastTransaction {
    valor: i32,
    tipo: String,
    descricao: String,
    realizada_em: String,
}

#[get("/{id}/extrato")]
async fn get_extract(db: web::Data<Database>, path: web::Path<i32>) -> HttpResponse {
    let last_transactions = db.get_last_transactions(path.into_inner());
    let mut is_first = true;
    let mut extract = Extract {
        saldo: None,
        ultimas_transacoes: None,
    };
    let now = SystemTime::now();
    let mut transactions = Vec::new();
    for last_transaction in last_transactions.iter() {
        if is_first {
            is_first = false;
            extract.saldo = Some(Balance {
                total: last_transaction.balance,
                data_extrato: iso8601(&now),
                limite: last_transaction.account_limit,
            });
        }
        transactions.push(LastTransaction {
            valor: last_transaction.amount,
            tipo: last_transaction.transaction_type.clone(),
            descricao: last_transaction.details.clone(),
            realizada_em: iso8601(&last_transaction.created_at),
        });
    }
    extract.ultimas_transacoes = Some(transactions);
    HttpResponse::Ok().json(extract)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/clientes")
            .service(get_clients)
            .service(get_client)
            .service(post_transaction)
            .service(get_extract),
    );
}
