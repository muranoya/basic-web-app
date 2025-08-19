use crate::models::User;
use anyhow::Result;
use sqlx::SqlitePool;

pub struct UserRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // 全てのクエリで使用する共通のSELECT句
    const SELECT_FIELDS: &'static str =
        "SELECT id, email, password, updated_at, created_at FROM users";

    /// メールアドレスでユーザーを検索
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(&format!("{} WHERE email = ?", Self::SELECT_FIELDS))
            .bind(email)
            .fetch_optional(self.pool)
            .await?;

        Ok(user)
    }

    /// IDでユーザーを検索
    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(&format!("{} WHERE id = ?", Self::SELECT_FIELDS))
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(user)
    }

    /// IDでユーザーを取得（存在することが前提）
    pub async fn get_by_id(&self, id: i64) -> Result<User> {
        let user = sqlx::query_as::<_, User>(&format!("{} WHERE id = ?", Self::SELECT_FIELDS))
            .bind(id)
            .fetch_one(self.pool)
            .await?;

        Ok(user)
    }

    /// 新しいユーザーを作成
    pub async fn create(&self, email: &str, password_hash: &str) -> Result<i64> {
        let result = sqlx::query("INSERT INTO users (email, password) VALUES (?, ?)")
            .bind(email)
            .bind(password_hash)
            .execute(self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// ユーザーのパスワードを更新
    pub async fn update_password(&self, id: i64, password_hash: &str) -> Result<()> {
        sqlx::query("UPDATE users SET password = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(password_hash)
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    /// ユーザーの存在確認（メールアドレス）
    pub async fn exists_by_email(&self, email: &str) -> Result<bool> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(self.pool)
            .await?;

        Ok(count.0 > 0)
    }
}
