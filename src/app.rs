pub mod usecase {
    use crate::{
        domain::{
            base::{TransactionUnit, UnitOfWork},
            entity::{CreateUserDto, User},
            repository::UserRepository,
        },
        infra::database::deadpool::PgDeadpoolUnit,
    };

    async fn validate_user_data<Cr>(_data: &CreateUserDto, _repo: &Cr)
    where
        Cr: UserRepository,
    {
        println!("validating...");
    }

    pub async fn create_user<Unit, Trx>(mut unit: Unit, data: CreateUserDto) -> Result<User, ()>
    where
        for<'t> Unit: UnitOfWork<Transaction<'t> = Trx>,
        Unit: UserRepository,
        Trx: TransactionUnit,
        Trx: UserRepository,
    {
        validate_user_data(&data, &unit).await;
        let user = User::try_from(data)?;

        let mut trx = unit.transaction().await.unwrap();

        trx.insert([user.clone()]).await.unwrap();
        UserRepository::insert(&mut trx, [user.clone()])
            .await
            .unwrap();

        trx.commit().await.unwrap();

        unit.insert([user.clone()]).await.unwrap();
        UserRepository::insert(&mut unit, [user]).await.unwrap();

        Err(())
    }

    pub async fn concrete_create_user(
        mut unit: PgDeadpoolUnit,
        data: CreateUserDto,
    ) -> Result<User, ()> {
        validate_user_data(&data, &unit).await;
        let user = User::try_from(data)?;

        let mut trx = unit.transaction().await.unwrap();

        trx.insert([user.clone()]).await.unwrap();
        UserRepository::insert(&mut trx, [user.clone()])
            .await
            .unwrap();

        trx.commit().await.unwrap();

        unit.insert([user.clone()]).await.unwrap();
        UserRepository::insert(&mut unit, [user]).await.unwrap();

        Err(())
    }
}
