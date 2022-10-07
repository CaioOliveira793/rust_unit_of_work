#![allow(dead_code)]

use app::usecase::customer::CreateCustomerRequest;
use domain::entities::customer::CreateCustomerData;
use uuid::Uuid;

use crate::{config::connection::create_client, infra::repositories::PgClient};

mod app;
mod config;
mod domain;
mod infra;

#[tokio::main]
async fn main() {
    let dto = CreateCustomerRequest {
        data: CreateCustomerData {
            id: Uuid::new_v4(),
            cpf: "23923824238".into(),
            name: "Rustacean".into(),
            phones: vec![],
        },
    };

    // let pool = create_pool();
    // let conn = pool.get().await.unwrap().client().to_owned();
    // let client = PgClient::new(conn);

    let mut client = PgClient::new(create_client());

    let customer = app::usecase::customer::concrete_create_customer(&mut client, dto)
        .await
        .unwrap();

    println!("customer {customer:?}");
}
