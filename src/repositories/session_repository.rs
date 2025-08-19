use crate::models::Session;
use anyhow::Result;
use sqlx::SqlitePool;

pub struct SessionRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SessionRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    const SELECT_FIELDS: &'static str = "SELECT id, user_id, uuid, csrf_token, issued_at, device_info, ip_address, created_at, updated_at FROM sessions";

    /// セッションを保存
    pub async fn create(&self, session: &Session) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO sessions (user_id, uuid, csrf_token, issued_at, device_info, ip_address) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(session.user_id)
        .bind(&session.uuid)
        .bind(&session.csrf_token)
        .bind(session.issued_at)
        .bind(&session.device_info)
        .bind(&session.ip_address)
        .execute(self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// セッションUUIDで検索
    pub async fn find_by_uuid(&self, session_uuid: &str) -> Result<Option<Session>> {
        let session =
            sqlx::query_as::<_, Session>(&format!("{} WHERE uuid = ?", Self::SELECT_FIELDS))
                .bind(session_uuid)
                .fetch_optional(self.pool)
                .await?;

        Ok(session)
    }

    /// セッションUUIDで削除（ログアウト）
    pub async fn delete_by_uuid(&self, session_uuid: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE uuid = ?")
            .bind(session_uuid)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    /// ユーザーIDで全セッションを削除（全デバイスログアウト）
    pub async fn delete_by_user_id(&self, user_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}
