# Unit of Work

A rust implementation of the [Unit of Work pattern](https://martinfowler.com/eaaCatalog/unitOfWork.html) using GAT (Generic associated types)

## Goal

- Full generic way to access the database through the `UnitOfWork`, `TransactionUnit`, and repository traits.
- Type safe way to create database transactions based on the trait methods, e.g. only possible to make a commit/roolback inside a transaction, invalidate the possibility of creating nested transactions.

Example code:

```rust
async fn validate_user_data<R>(data: &CreateUserDto, repo: &R)
where
	R: UserRepository,
{
	unimplemented!()
}

async fn create_user<DB>(req: CreateUserRequest) -> Result<User, ()>
where
	DB: UnitOfWork,
	DB: TransactionUnit,
	DB: UserRepository,
{
	let mut unit = create_unit_somehow::<DB>().await?;

	validate_user_data::<DB>(&req.data, &unit).await?;
	let user = User::try_from(req.data)?;

	let mut trx = unit.transaction().await?;

	trx.insert([user.clone()]).await?;
	trx.insert([user.clone()]).await?;

	trx.commit().await?;

	unit.insert([user.clone()]).await?;

	UserRepository::insert(&mut unit, [user])
		.await
		?;

	Err(())
}
```

## Status

- Currently usable providing the concrete type that implements the `UnitOfWork` trait

## Todo

- [ ] Create a working example with the `tokio_postgres` client
- [ ] Implement more database connections
  - [ ] `mysql_async`
  - [ ] `rusqlite`
