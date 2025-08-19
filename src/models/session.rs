use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub uuid: String,
    pub csrf_token: String,
    pub issued_at: DateTime<Utc>,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: i64, device_info: Option<String>, ip_address: Option<String>) -> Self {
        let session_uuid = Uuid::new_v4().to_string();
        let csrf_token_uuid = Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            id: 0, // will be set by database
            user_id,
            uuid: session_uuid,
            csrf_token: csrf_token_uuid,
            issued_at: now,
            device_info,
            ip_address,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn get_session_uuid(&self) -> &str {
        &self.uuid
    }
}
