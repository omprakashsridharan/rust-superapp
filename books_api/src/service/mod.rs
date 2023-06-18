pub mod book_created_producer;
use crate::dto::{Book, BookBuilder, BookBuilderError};
use crate::repository::{Repository, RepositoryError};
use thiserror::Error;
use tracing::{info_span, Instrument};

use self::book_created_producer::{BookCreatedProducer, BookCreatedProducerError};

trait Async: Send + Sync {}

#[derive(Clone)]
pub struct Service {
    repository: Repository,
    book_created_producer: BookCreatedProducer,
}

impl Async for Service {}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Repository error")]
    RepositoryError(#[from] RepositoryError),

    #[error("BookBuilder error")]
    BookBuilderError(#[from] BookBuilderError),

    #[error("BookCreatedProducer error")]
    BookCreatedProducer(#[from] BookCreatedProducerError),
}

impl Service {
    pub fn new(repository: Repository, book_created_producer: BookCreatedProducer) -> Self {
        Self {
            repository,
            book_created_producer,
        }
    }

    pub async fn create_and_publish_book(
        &self,
        title: String,
        isbn: String,
    ) -> Result<Book, ServiceError> {
        let span = info_span!("create_and_publish_book repository create_book");
        let created_book_model = async move {
            let created_book_model = self
                .repository
                .create_book(title, isbn)
                .await
                .map_err(|e| ServiceError::RepositoryError(e));
            return created_book_model;
        }
        .instrument(span)
        .await?;
        tokio::spawn(
            self.book_created_producer
                .publish_created_book(
                    created_book_model.id,
                    created_book_model.title.clone(),
                    created_book_model.isbn.clone(),
                )
                .instrument(info_span!("publish_created_book")),
        )
        .await;

        let book = BookBuilder::default()
            .id(created_book_model.id)
            .title(created_book_model.title)
            .isbn(created_book_model.isbn)
            .build()
            .map_err(|e| ServiceError::BookBuilderError(e))?;
        Ok(book)
    }
}
