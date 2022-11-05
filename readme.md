# Unit of Work

A rust implementation of the [Unit of Work pattern](https://martinfowler.com/eaaCatalog/unitOfWork.html) using GAT (Generic associated type)

## Goal

- Fully generic way to access the database through `DbUnit`, `TransactionUnit`, and repository traits.
- Type safe way to create database transactions based on the trait methods, e.g. only possible to make a commit/roolback inside a transaction, invalidate the possibility of creating nested transactions.

Example code:

```rust
trait UserRepository: Repository {
	async fn insert<I>(&mut self, user: I) -> Result<(), RepositoryError>
	where
		I: IntoIterator<Item = User>;

	async fn find(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError>;
}

async fn validate_user<R>(data: &User, repo: &R) -> Result<(), ValidationError>
where
	R: UserRepository,
{
	unimplemented!()
}

async fn create_user<Unit, Trx>(mut unit: Unit, req: CreateUserRequest) -> Result<User, Error>
where
	for<'t> Unit: DbUnit<Transaction<'t> = Trx>,
	Unit: UserRepository,
	Trx: TransactionUnit,
	Trx: UserRepository,
{
	let user = User::try_from(req.data)?;
	validate_user(&user, &unit).await?;

	let mut trx = unit.transaction().await?;

	trx.insert([user.clone()]).await?;
	UserRepository::insert(trx, [user.clone()]).await?;

	trx.commit().await?;

	unit.insert([user.clone()]).await?;
	UserRepository::insert(&mut unit, [user.clone()]).await?;

	Ok(user)
}
```

## Status

Currently usable providing the concrete type that implements the `DbUnit` trait

At the moment, a fully generic function is not possible due an issue with HRTB

### Higher-Rank Trait Bound issue investigation

- A [great article](https://lucumr.pocoo.org/2022/9/11/abstracting-over-ownership/) that explain the issues encountered in this crate
- [Lifetime inference problem](https://users.rust-lang.org/t/hrtb-on-multiple-generics/34255)
- [A value does not implement some type with the specified lifetimes, when actually is implemented for any lifetime](https://github.com/rust-lang/rust/issues/70263)
- [Inconsistent behaviour with GAT](https://github.com/rust-lang/rust/issues/99548)
- [RFC for bounded universal quantification for lifetimes](https://github.com/rust-lang/rfcs/pull/3261)

## Todo

- Implement more database connections/pools
  - `bb8-postgres`
  - `mysql_async`
  - `rusqlite`
- create trait `DbDriver` to have a common interface when implementing the repositories
  - implement the repositories through functions generic over `DbDriver`
