use thiserror::Error;
use crate::dto::{Book, BookBuilder, BookBuilderError};
use crate::repository::{Repository, RepositoryError};


pub struct Service {
    repository: Repository,
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Repository error")]
    RepositoryError(#[from] RepositoryError),

    #[error("BookBuilder error")]
    BookBuilderError(#[from] BookBuilderError),
}

impl Service {
    pub fn new(repository: Repository) -> Self {
        Self {
            repository
        }
    }

    pub async fn create_and_publish_book(&self, title: String, isbn: String) -> Result<Book, ServiceError> {
        let created_book_model = self.repository.create_book(title, isbn).await.map_err(|e| ServiceError::RepositoryError(e))?;
        let book = BookBuilder::default().id(created_book_model.id).title(created_book_model.title).isbn(created_book_model.isbn).build().map_err(|e| ServiceError::BookBuilderError(e))?;
        Ok(book)
    }
}

