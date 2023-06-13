use std::sync::Arc;
use derive_builder::Builder;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr};
use sea_orm::ActiveValue::Set;
use thiserror::Error;
use crate::entity::book::{Model as BookModel, ActiveModel as BookActiveModel};

#[derive(Builder)]
pub struct Repository {
    database_connection: Arc<DatabaseConnection>,
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error")]
    DatabaseError(#[from] DbErr)
}

impl Repository {
    pub async fn create_book(&self, title: String, isbn: String) -> Result<BookModel, RepositoryError> {
        let created_book = BookActiveModel {
            title: Set(title),
            isbn: Set(isbn),
            ..Default::default()
        };
        created_book.insert(self.database_connection.as_ref()).await.map_err(|e| RepositoryError::DatabaseError(e))
    }
}