use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>, 
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateCategoryPayload {
    pub name: String,
    pub description: Option<String>,
}


#[derive(Deserialize)]
pub struct UpdateCategoryPayload {
    pub name: Option<String>,
    pub description: Option<String>,
}