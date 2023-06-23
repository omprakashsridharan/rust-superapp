use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use tracing::log;

pub async fn get_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(10))
        .max_lifetime(Duration::from_secs(10))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);
    return Database::connect(opt).await;
}

#[cfg(test)]
mod tests {
    use crate::get_connection;
    use sea_orm::{ConnectionTrait, DatabaseBackend, QueryResult, Statement};
    use testcontainers::{clients, images};

    #[tokio::test]
    async fn test_database_connection() {
        let docker = clients::Cli::default();
        let database = images::postgres::Postgres::default();
        let node = docker.run(database);
        let connection_string = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port_ipv4(5432)
        );
        let database_connection = get_connection(connection_string).await.unwrap();
        let query_res: Option<QueryResult> = database_connection
            .query_one(Statement::from_string(
                DatabaseBackend::Postgres,
                "SELECT 1;".to_owned(),
            ))
            .await
            .unwrap();
        let query_res = query_res.unwrap();
        let value: i32 = query_res.try_get_by_index(0).unwrap();
        assert_eq!(1, value);
    }
}
