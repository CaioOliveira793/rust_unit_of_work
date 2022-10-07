use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime, SslMode};
use tokio_postgres::{Client, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;

fn connection_config() -> Config {
    let host = std::env::var("DATABASE_HOST").expect("Missing env var DATABASE_HOST");
    let database = std::env::var("DATABASE_NAME").expect("Missing env var DATABASE_NAME");
    let port: u16 = std::env::var("DATABASE_PORT")
        .expect("Missing env var DATABASE_PORT")
        .parse()
        .expect("Invalid env var DATABASE_PORT");
    let user = std::env::var("DATABASE_USER").expect("Missing env var DATABASE_USER");
    let password = std::env::var("DATABASE_PASSWORD").expect("Missing env var DATABASE_PASSWORD");

    let mut cfg = Config::new();
    cfg.dbname = Some(database);
    cfg.user = Some(user);
    cfg.password = Some(password);
    cfg.port = Some(port);
    cfg.host = Some(host);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg.application_name = Some("lf_import_lambda".into());
    cfg.ssl_mode = Some(SslMode::Prefer);
    cfg
}

#[allow(dead_code)]
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

pub fn create_pool() -> Pool {
    let cfg = connection_config();

    cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}

pub fn create_client() -> Client {
    // TODO: create tokio_postgres client
    todo!()
}
