use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct IdPayload {
    pub id: i32,
}

#[derive(Serialize)]
pub struct FilterResponse<T> {
    pub total_count: i32,
    pub data: Vec<T>,
}

