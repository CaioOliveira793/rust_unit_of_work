pub mod customer {
    use crate::{
        domain::{
            base::{TransactionUnit, UnitOfWork},
            entities::customer::{CreateCustomerData, Customer},
            repositories::CustomerRepository,
        },
        infra::repositories::PgUnit,
    };

    pub struct CreateCustomerRequest {
        pub data: CreateCustomerData,
    }

    async fn validate_customer_data<Cr>(_data: &CreateCustomerData, _repo: &Cr)
    where
        Cr: CustomerRepository,
    {
        println!("validating...");
    }

    // pub async fn create_customer<'db, Db, Trx>(
    //     mut unit: Db,
    //     req: CreateCustomerRequest,
    // ) -> Result<Customer, ()>
    // where
    //     Db: UnitOfWork<Transaction<'db> = Db>,
    //     Db: TransactionUnit,
    //     Db: CustomerRepository,
    //     Db: 'db,
    // {
    //     validate_customer_data::<Db>(&req.data, &unit).await;
    //     let customer = Customer::try_from(req.data)?;

    //     {
    //         let mut trx = unit.transaction().await.unwrap();

    //         trx.insert([customer.clone()]).await.unwrap();
    //         trx.insert([customer.clone()]).await.unwrap();

    //         trx.commit().await.unwrap();
    //     }

    //     unit.insert([customer.clone()]).await.unwrap();

    //     CustomerRepository::insert(&mut unit, [customer])
    //         .await
    //         .unwrap();

    //     Err(())
    // }

    pub async fn concrete_create_customer(
        unit: &mut PgUnit,
        req: CreateCustomerRequest,
    ) -> Result<Customer, ()> {
        validate_customer_data(&req.data, unit).await;
        let customer = Customer::try_from(req.data)?;

        let mut trx = unit.transaction().await.unwrap();

        trx.insert([customer.clone()]).await.unwrap();
        trx.insert([customer.clone()]).await.unwrap();

        trx.commit().await.unwrap();

        unit.insert([customer.clone()]).await.unwrap();

        CustomerRepository::insert(unit, [customer]).await.unwrap();

        Err(())
    }
}
