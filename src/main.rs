#![allow(dead_code)]

use domain::entities::customer::CreateCustomerData;
use uuid::Uuid;

use crate::{config::connection::create_client, infra::repositories::PgUnit};

mod app;
mod config;
mod domain;
mod infra;

#[tokio::main]
async fn main() {
    let data = CreateCustomerData {
        id: Uuid::new_v4(),
        cpf: "23923824238".into(),
        name: "Rustacean".into(),
        phones: vec![],
    };

    // let pool = create_pool();
    // let conn = pool.get().await.unwrap().client().to_owned();
    // let client = PgUnit::new(conn);

    // Impratical, PgClient does no implement Clone
    // only the pool connection
    let client = PgUnit::new(create_client());

    let customer = app::usecase::customer::concrete_create_customer(client, data.clone())
        .await
        .unwrap();
    println!("customer {customer:?}");

    // let client = PgUnit::new(create_client());
    // let customer = app::usecase::customer::create_customer::<PgUnit, PgTrxUnit>(client, data)
    //     .await
    //     .unwrap();
    // println!("customer {customer:?}");
}
