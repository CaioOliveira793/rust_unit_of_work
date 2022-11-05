#![allow(dead_code)]

use domain::entity::CreateUserDto;
use uuid::Uuid;

use crate::config::connection;

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

    let pool = connection::create_deadpool();
    let client = pool.get().await.unwrap();

    let user = app::usecase::concrete_create_user(client, data.clone())
        .await
        .unwrap();
    println!("user {user:?}");

    // let client = pool.get().await.unwrap();
    // let user = app::usecase::create_user(client, data).await.unwrap();
    // println!("user {user:?}");
}
