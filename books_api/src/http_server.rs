use crate::service::{Service, ServiceError};
use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tracing::error;

pub async fn start_http_server(service: Service) {
    let books_router = Router::new().route("/", post(create_book));
    let api_router = Router::new().nest("/books", books_router);
    let app = Router::new()
        .nest("/api", api_router)
        .layer(opentelemetry_tracing_layer())
        .layer(Extension(service));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        error!("Service Error {}", self);
        let (status, error_message) = match self {
            ServiceError::RepositoryError(re) => {
                (StatusCode::INTERNAL_SERVER_ERROR, re.to_string())
            }
            e => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateBookRequest {
    title: String,
    isbn: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateBookResponse {
    id: i32,
    title: String,
    isbn: String,
}

async fn create_book(
    Extension(service): Extension<Service>,
    Json(create_book_request): Json<CreateBookRequest>,
) -> impl IntoResponse {
    let created_book_result = service
        .create_and_publish_book(create_book_request.title, create_book_request.isbn)
        .await;
    return match created_book_result {
        Ok(created_book) => (
            StatusCode::CREATED,
            Json(json!(CreateBookResponse {
                id: created_book.id,
                title: created_book.title,
                isbn: created_book.isbn,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    };
}
