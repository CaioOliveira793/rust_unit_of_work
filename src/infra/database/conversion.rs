use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use tokio_postgres::Error;
use uuid::Uuid;

use crate::domain::{base::RepositoryError, entities::customer::Customer, types::Phone};

impl From<Error> for RepositoryError {
    fn from(err: Error) -> Self {
        if let Some(db_err) = err.as_db_error() {
            return RepositoryError::Db(db_err.clone());
        }

        RepositoryError::Unknown(err.into())
    }
}

impl From<tokio_postgres::Row> for Phone {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            verified: row.get("verified"),
            number: row.get("number"),
        }
    }
}

pub fn map_row_by<T, Rows>(id_idx: &str, rows: Rows) -> HashMap<Uuid, Vec<T>>
where
    T: From<tokio_postgres::Row>,
    Rows: IntoIterator<Item = tokio_postgres::Row>,
{
    let mut map: HashMap<Uuid, Vec<T>> = HashMap::new();
    for row in rows {
        let id: Uuid = row.get(id_idx);
        let value: T = row.into();
        if let Some(values) = map.get_mut(&id) {
            values.push(value);
        } else {
            map.insert(id, vec![value]);
        }
    }
    map
}

pub fn customer_from_row(
    customer_rows: Vec<tokio_postgres::Row>,
    phone_rows: Vec<tokio_postgres::Row>,
) -> Vec<Customer> {
    let mut phones_map: HashMap<Uuid, Vec<Phone>> = map_row_by("customer_id", phone_rows);

    customer_rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let updated: Option<NaiveDateTime> = row.get("updated");
            Customer {
                id,
                cpf: row.get("cpf"),
                name: row.get("name"),
                phones: phones_map.remove(&id).unwrap_or_default(),
                created: DateTime::from_utc(row.get("created"), Utc),
                updated: updated.map(|naive| DateTime::from_utc(naive, Utc)),
            }
        })
        .collect()
}
