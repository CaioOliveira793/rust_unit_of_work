#![feature(generic_associated_types)]
#![allow(dead_code)]

use app::usecase::customer::CreateCustomerRequest;
use domain::entities::customer::CreateCustomerData;
use uuid::Uuid;

mod app;
mod config;
mod domain;
mod infra;

#[tokio::main]
async fn main() {
    let dto = CreateCustomerRequest {
        data: CreateCustomerData {
            id: Uuid::new_v4(),
            cpf: "08177663593".into(),
            name: "Caio Oliveira".into(),
            phones: vec![],
        },
    };

    let customer = app::usecase::customer::concrete_create_customer(dto)
        .await
        .unwrap();

    println!("customer {customer:?}");
}
