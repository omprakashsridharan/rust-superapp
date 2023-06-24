pub mod book_created_producer;
use self::book_created_producer::{BookCreatedProducer, BookCreatedProducerError};
use crate::dto::{Book, BookBuilder, BookBuilderError};
use crate::repository::{Repository, RepositoryError};
use std::sync::Arc;
use thiserror::Error;
use tracing::{info_span, Instrument};

#[derive(Clone)]
pub struct Service {
    repository: Repository,
    book_created_producer: Arc<BookCreatedProducer>,
}

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
            book_created_producer: Arc::new(book_created_producer),
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
        let producer = self.book_created_producer.clone();
        let created_book_id = created_book_model.clone().id;
        let created_book_title = created_book_model.clone().title;
        let created_book_isbn = created_book_model.clone().isbn;
        let _ = tokio::task::spawn(async move {
            let _ = producer
                .publish_created_book(created_book_id, created_book_title, created_book_isbn)
                .await;
        })
        .instrument(info_span!("publish_created_book"));

        let book = BookBuilder::default()
            .id(created_book_model.id)
            .title(created_book_model.title)
            .isbn(created_book_model.isbn)
            .build()
            .map_err(|e| ServiceError::BookBuilderError(e))?;
        Ok(book)
    }
}
