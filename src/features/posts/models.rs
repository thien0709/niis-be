use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub content_markdown: String,
    pub cover_image_url: Option<String>,
    pub published: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreatePostPayload {
    pub title: String,
    pub content_markdown: String,
    pub cover_image_url: Option<String>,
    pub category_id: Option<Uuid>,
}