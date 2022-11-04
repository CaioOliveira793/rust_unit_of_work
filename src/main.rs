#![allow(dead_code)]

use domain::entity::CreateUserDto;
use uuid::Uuid;

use crate::{config::connection::create_client, infra::database::client::PgUnit};

mod app;
mod config;
mod domain;
mod infra;

#[tokio::main]
async fn main() {
    let data = CreateUserDto {
        id: Uuid::new_v4(),
        email: "rustac@email.com".into(),
        name: "Rustacean".into(),
        phones: vec![],
    };

    // let pool = create_pool();
    // let conn = pool.get().await.unwrap().client().to_owned();
    // let client = PgUnit::new(conn);

    // Impratical, PgClient does no implement Clone
    // only the pool connection
    let client = PgUnit::new(create_client());

    let user = app::usecase::concrete_create_user(client, data.clone())
        .await
        .unwrap();
    println!("user {user:?}");

    // let client = PgUnit::new(create_client());
    // let user = app::usecase::create_user::<PgUnit, PgTrxUnit>(client, data)
    //     .await
    //     .unwrap();
    // println!("user {user:?}");
}
