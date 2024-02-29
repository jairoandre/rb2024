use crate::models::*;
use crate::AppState;
use actix_web::{
    get, post,
    web::{scope, Data, Json, Path, ServiceConfig},
    HttpResponse,
};
use chrono::{DateTime, Utc};
use sqlx::Acquire;
use std::time::SystemTime;

fn iso8601(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.to_rfc3339())
}

#[post("/{id}/transacoes")]
async fn post_transaction(
    state: Data<AppState>,
    path: Path<i32>,
    payload: Json<TransactionPayload>,
) -> HttpResponse {
    let client_id = path.into_inner();
    let amount = match payload.tipo.as_str() {
        "c" => payload.valor,
        "d" => payload.valor * -1,
        _ => return HttpResponse::UnprocessableEntity().body("Invalid operation"),
    };
    let desc = match &payload.descricao {
        Some(d) => d.clone(),
        None => return HttpResponse::UnprocessableEntity().body("Description is mandatory"),
    };
    if desc.is_empty() {
        return HttpResponse::UnprocessableEntity().body("Description is mandatory");
    }
    let mut conn = state.db.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();
    // lock the row for updates
    match sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1 FOR UPDATE")
        .bind(client_id)
        .fetch_one(&mut *tx)
        .await
    {
        Ok(c) => {
            let new_balance = c.balance + amount;
            if (new_balance + c.account_limit) < 0 {
                return HttpResponse::UnprocessableEntity().body("Limit reached");
            }
            let _ = sqlx::query("UPDATE clients SET balance = $1 WHERE id = $2")
                .bind(new_balance)
                .bind(c.id)
                .execute(&mut *tx)
                .await
                .unwrap();
            match sqlx::query("INSERT INTO transactions (client_id, amount, transaction_type, details) VALUES ($1, $2, $3, $4)")
                    .bind(c.id)
                    .bind(payload.valor)
                    .bind(payload.tipo.clone())
                    .bind(desc)
                    .execute(&mut *tx)
                    .await {
                        Ok(_) => {
                            let _ = tx.commit().await.unwrap();
                            let response = TransactionResponse {
                                limite: c.account_limit,
                                saldo: new_balance,
                            };
                            HttpResponse::Ok().json(response)
                        },
                        Err(_) => {
                            let _ = tx.rollback().await.unwrap();
                            HttpResponse::UnprocessableEntity().body("Internal server error")
                        }
                    }
        }
        Err(_) => {
            let _ = tx.rollback().await.unwrap();
            HttpResponse::UnprocessableEntity().body("Internal server error")
        }
    }
}

static LAST_TRANSACTION_SQL: &str = "
SELECT
tx.amount AS valor,
tx.transaction_type AS tipo,
tx.details AS descricao,
TO_CHAR(tx.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MSZ') AS realizada_em
FROM transactions tx WHERE tx.client_id = $1 ORDER BY tx.created_at DESC LIMIT 10;
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
        Err(e) => {
            println!("{}", e);
            HttpResponse::NotFound().body("Client not found")
        }
    }
}

pub fn init_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/clientes")
            .service(post_transaction)
            .service(get_extract),
    );
}
