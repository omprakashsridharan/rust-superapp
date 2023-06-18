use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Builder, Debug)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub isbn: String,
}
