pub mod connection {
    use std::time::Duration;

    use bb8_postgres::{bb8, PostgresConnectionManager};
    use deadpool_postgres::{ManagerConfig, RecyclingMethod};
    use tokio_postgres_rustls::MakeRustlsConnect;

    fn connection_config() -> tokio_postgres::Config {
        let host = std::env::var("DATABASE_HOST").expect("Missing env var DATABASE_HOST");
        let database = std::env::var("DATABASE_NAME").expect("Missing env var DATABASE_NAME");
        let port: u16 = std::env::var("DATABASE_PORT")
            .expect("Missing env var DATABASE_PORT")
            .parse()
            .expect("Invalid env var DATABASE_PORT");
        let user = std::env::var("DATABASE_USER").expect("Missing env var DATABASE_USER");
        let password =
            std::env::var("DATABASE_PASSWORD").expect("Missing env var DATABASE_PASSWORD");

        let mut cfg = tokio_postgres::Config::new();
        cfg.dbname(&database);
        cfg.user(&user);
        cfg.password(password);
        cfg.port(port);
        cfg.host(&host);
        cfg.connect_timeout(Duration::from_millis(5000));
        cfg.application_name("UoW test".into());
        cfg.ssl_mode(tokio_postgres::config::SslMode::Require);
        cfg
    }

    fn tls_config() -> MakeRustlsConnect {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let tls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        MakeRustlsConnect::new(tls_config)
    }

    fn deadpool_config(cfg: tokio_postgres::Config) -> deadpool_postgres::Config {
        let host = std::env::var("DATABASE_HOST").expect("Missing env var DATABASE_HOST");

        let mut config = deadpool_postgres::Config::new();
        config.dbname = cfg.get_dbname().map(ToOwned::to_owned);
        config.user = cfg.get_user().map(ToOwned::to_owned);
        config.password = cfg
            .get_password()
            .map(|pass| String::from_utf8(pass.into()).unwrap());
        config.port = cfg.get_ports().iter().nth(0).copied();
        config.host = Some(host);
        config.connect_timeout = cfg.get_connect_timeout().cloned();
        config.application_name = Some("UoW test".into());
        config.ssl_mode = Some(deadpool_postgres::SslMode::Require);
        config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        config
    }

    pub fn create_pg_deadpool() -> deadpool_postgres::Pool {
        let config = connection_config();
        let tls = tls_config();

        deadpool_config(config)
            .create_pool(Some(deadpool_postgres::Runtime::Tokio1), tls)
            .unwrap()
    }

    pub type PgBb8pool = bb8::Pool<PostgresConnectionManager<MakeRustlsConnect>>;

    pub async fn create_pg_bb8pool() -> PgBb8pool {
        let config = connection_config();
        let tls = tls_config();
        let manager = PostgresConnectionManager::new(config, tls);

        bb8::Pool::builder()
            .max_size(5)
            .build(manager)
            .await
            .unwrap()
    }
}
