macro_rules! read_prop {
    ($propfn:ident, $transf:ident, $out:ty) => {
        #[allow(dead_code)]
        pub fn $propfn(&self) -> &$out {
            return &self.$transf
        }
    };
}

pub mod customer {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use crate::domain::types::Phone;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Customer {
        pub id: Uuid,
        pub name: String,
        pub cpf: String,
        pub phones: Vec<Phone>,
        pub created: DateTime<Utc>,
        pub updated: Option<DateTime<Utc>>,
    }

    impl Customer {
        read_prop!(ident, id, Uuid);
        // read_prop!(name, name, String);
        // read_prop!(cpf, cpf, String);
        // read_prop!(phones, phones, Vec<Phone>);
        // read_prop!(created, created, DateTime<Utc>);
        // read_prop!(updated, updated, Option<DateTime<Utc>>);
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CreateCustomerData {
        pub id: Uuid,
        pub name: String,
        pub cpf: String,
        pub phones: Vec<Phone>,
    }

    impl TryFrom<CreateCustomerData> for Customer {
        type Error = ();

        fn try_from(data: CreateCustomerData) -> Result<Self, Self::Error> {
            if data.phones.is_empty() {
                return Err(());
            }

            Ok(Self {
                id: data.id,
                cpf: data.cpf,
                name: data.name,
                phones: data.phones,
                created: Utc::now(),
                updated: None,
            })
        }
    }
}
