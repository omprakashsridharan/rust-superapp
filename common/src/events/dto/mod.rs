use apache_avro::AvroSchema;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Builder, Clone, Debug, AvroSchema)]
pub struct CreatedBook {
    id: i32,
    title: String,
    isbn: String,
}
