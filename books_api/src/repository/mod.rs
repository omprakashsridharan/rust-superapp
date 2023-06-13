use std::sync::Arc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr};
use sea_orm::ActiveValue::Set;
use thiserror::Error;
use migration::{Migrator, MigratorTrait};
use crate::entity::book::{Model as BookModel, ActiveModel as BookActiveModel};


pub struct Repository {
    database_connection: Arc<DatabaseConnection>,
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error")]
    DatabaseError(#[from] DbErr)
}

impl Repository {
    pub async fn new(database_connection: DatabaseConnection) -> Result<Self, RepositoryError> {
        Migrator::up(&database_connection, None).await.map_err(|e| RepositoryError::DatabaseError(e))?;
        Ok(Self {
            database_connection: Arc::new(database_connection)
        })
    }

    pub async fn create_book(&self, title: String, isbn: String) -> Result<BookModel, RepositoryError> {
        let created_book = BookActiveModel {
            title: Set(title),
            isbn: Set(isbn),
            ..Default::default()
        };
        created_book.insert(self.database_connection.as_ref()).await.map_err(|e| RepositoryError::DatabaseError(e))
    }
}


#[cfg(test)]
mod tests {
    use testcontainers::{clients, images};
    use database::get_connection;
    use crate::repository::Repository;

    #[tokio::test]
    async fn test_create_book() {
        let docker = clients::Cli::default();
        let database = images::postgres::Postgres::default();
        let node = docker.run(database);
        let connection_string = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port_ipv4(5432)
        );
        let database_connection = get_connection(connection_string).await.unwrap();
        let repository = Repository::new(database_connection.clone()).await.unwrap();
        let title = "TITLE".to_string();
        let isbn = "ISBN".to_string();
        let created_book = repository.create_book(title.clone(), isbn.clone()).await.unwrap();
        assert_eq!(created_book.title, title.clone());
        assert_eq!(created_book.isbn, isbn.clone());
    }
}