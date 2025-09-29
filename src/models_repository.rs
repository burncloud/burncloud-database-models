use async_trait::async_trait;
use sqlx::{Database, Pool, Row};
use uuid::Uuid;
use crate::models_table::{ModelsTable, InstalledModelsTable};

/// Models 表的数据库操作接口
#[async_trait]
pub trait ModelsRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    // === Models 表操作 ===

    /// 获取所有模型
    async fn get_all_models(&self) -> Result<Vec<ModelsTable>, Self::Error>;

    /// 根据ID获取模型
    async fn get_model_by_id(&self, id: Uuid) -> Result<Option<ModelsTable>, Self::Error>;

    /// 根据名称获取模型
    async fn get_model_by_name(&self, name: &str) -> Result<Option<ModelsTable>, Self::Error>;

    /// 创建新模型
    async fn create_model(&self, model: &ModelsTable) -> Result<(), Self::Error>;

    /// 更新模型
    async fn update_model(&self, model: &ModelsTable) -> Result<(), Self::Error>;

    /// 删除模型
    async fn delete_model(&self, id: Uuid) -> Result<(), Self::Error>;

    /// 搜索模型
    async fn search_models(&self, query: &str, limit: Option<i64>) -> Result<Vec<ModelsTable>, Self::Error>;

    /// 根据类型获取模型
    async fn get_models_by_type(&self, model_type: &str) -> Result<Vec<ModelsTable>, Self::Error>;

    /// 根据提供商获取模型
    async fn get_models_by_provider(&self, provider: &str) -> Result<Vec<ModelsTable>, Self::Error>;

    /// 获取官方模型
    async fn get_official_models(&self) -> Result<Vec<ModelsTable>, Self::Error>;

    /// 增加下载次数
    async fn increment_download_count(&self, id: Uuid) -> Result<(), Self::Error>;

    // === Installed Models 表操作 ===

    /// 获取所有已安装模型
    async fn get_all_installed_models(&self) -> Result<Vec<InstalledModelsTable>, Self::Error>;

    /// 根据模型ID获取已安装模型
    async fn get_installed_model_by_model_id(&self, model_id: Uuid) -> Result<Option<InstalledModelsTable>, Self::Error>;

    /// 安装模型
    async fn install_model(&self, installed_model: &InstalledModelsTable) -> Result<(), Self::Error>;

    /// 更新已安装模型
    async fn update_installed_model(&self, installed_model: &InstalledModelsTable) -> Result<(), Self::Error>;

    /// 卸载模型
    async fn uninstall_model(&self, model_id: Uuid) -> Result<(), Self::Error>;

    /// 根据状态获取已安装模型
    async fn get_installed_models_by_status(&self, status: &str) -> Result<Vec<InstalledModelsTable>, Self::Error>;

    /// 更新模型使用情况
    async fn update_model_usage(&self, model_id: Uuid) -> Result<(), Self::Error>;

    /// 获取模型和安装信息的联合查询
    async fn get_models_with_install_info(&self) -> Result<Vec<(ModelsTable, Option<InstalledModelsTable>)>, Self::Error>;
}

/// SQLite 数据库的 Models Repository 实现
pub struct SqliteModelsRepository<DB: Database> {
    pool: Pool<DB>,
}

impl<DB: Database> SqliteModelsRepository<DB> {
    pub fn new(pool: Pool<DB>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModelsRepository for SqliteModelsRepository<sqlx::Sqlite> {
    type Error = sqlx::Error;

    async fn get_all_models(&self) -> Result<Vec<ModelsTable>, Self::Error> {
        let models = sqlx::query_as::<_, ModelsTable>(
            "SELECT * FROM models ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(models)
    }

    async fn get_model_by_id(&self, id: Uuid) -> Result<Option<ModelsTable>, Self::Error> {
        let model = sqlx::query_as::<_, ModelsTable>(
            "SELECT * FROM models WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(model)
    }

    async fn get_model_by_name(&self, name: &str) -> Result<Option<ModelsTable>, Self::Error> {
        let model = sqlx::query_as::<_, ModelsTable>(
            "SELECT * FROM models WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(model)
    }

    async fn create_model(&self, model: &ModelsTable) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            INSERT INTO models (
                id, name, display_name, description, version, model_type,
                size_category, file_size, provider, license, tags, languages,
                file_path, checksum, download_url, config, rating,
                download_count, is_official, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&model.id)
        .bind(&model.name)
        .bind(&model.display_name)
        .bind(&model.description)
        .bind(&model.version)
        .bind(&model.model_type)
        .bind(&model.size_category)
        .bind(model.file_size)
        .bind(&model.provider)
        .bind(&model.license)
        .bind(&model.tags)
        .bind(&model.languages)
        .bind(&model.file_path)
        .bind(&model.checksum)
        .bind(&model.download_url)
        .bind(&model.config)
        .bind(model.rating)
        .bind(model.download_count)
        .bind(model.is_official)
        .bind(model.created_at)
        .bind(model.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_model(&self, model: &ModelsTable) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            UPDATE models SET
                display_name = ?, description = ?, version = ?, model_type = ?,
                size_category = ?, file_size = ?, provider = ?, license = ?,
                tags = ?, languages = ?, file_path = ?, checksum = ?,
                download_url = ?, config = ?, rating = ?, download_count = ?,
                is_official = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&model.display_name)
        .bind(&model.description)
        .bind(&model.version)
        .bind(&model.model_type)
        .bind(&model.size_category)
        .bind(model.file_size)
        .bind(&model.provider)
        .bind(&model.license)
        .bind(&model.tags)
        .bind(&model.languages)
        .bind(&model.file_path)
        .bind(&model.checksum)
        .bind(&model.download_url)
        .bind(&model.config)
        .bind(model.rating)
        .bind(model.download_count)
        .bind(model.is_official)
        .bind(model.updated_at)
        .bind(&model.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_model(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM models WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn search_models(&self, query: &str, limit: Option<i64>) -> Result<Vec<ModelsTable>, Self::Error> {
        let search_pattern = format!("%{}%", query);
        let limit_clause = limit.unwrap_or(50);

        let models = sqlx::query_as::<_, ModelsTable>(
            r#"
            SELECT * FROM models
            WHERE name LIKE ? OR display_name LIKE ? OR description LIKE ?
            ORDER BY created_at DESC
            LIMIT ?
            "#
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit_clause)
        .fetch_all(&self.pool)
        .await?;

        Ok(models)
    }

    async fn get_models_by_type(&self, model_type: &str) -> Result<Vec<ModelsTable>, Self::Error> {
        let models = sqlx::query_as::<_, ModelsTable>(
            "SELECT * FROM models WHERE model_type = ? ORDER BY created_at DESC"
        )
        .bind(model_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(models)
    }

    async fn get_models_by_provider(&self, provider: &str) -> Result<Vec<ModelsTable>, Self::Error> {
        let models = sqlx::query_as::<_, ModelsTable>(
            "SELECT * FROM models WHERE provider = ? ORDER BY created_at DESC"
        )
        .bind(provider)
        .fetch_all(&self.pool)
        .await?;

        Ok(models)
    }

    async fn get_official_models(&self) -> Result<Vec<ModelsTable>, Self::Error> {
        let models = sqlx::query_as::<_, ModelsTable>(
            "SELECT * FROM models WHERE is_official = true ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(models)
    }

    async fn increment_download_count(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query(
            "UPDATE models SET download_count = download_count + 1, updated_at = ? WHERE id = ?"
        )
        .bind(chrono::Utc::now())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_all_installed_models(&self) -> Result<Vec<InstalledModelsTable>, Self::Error> {
        let installed_models = sqlx::query_as::<_, InstalledModelsTable>(
            "SELECT * FROM installed_models ORDER BY installed_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(installed_models)
    }

    async fn get_installed_model_by_model_id(&self, model_id: Uuid) -> Result<Option<InstalledModelsTable>, Self::Error> {
        let installed_model = sqlx::query_as::<_, InstalledModelsTable>(
            "SELECT * FROM installed_models WHERE model_id = ?"
        )
        .bind(model_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(installed_model)
    }

    async fn install_model(&self, installed_model: &InstalledModelsTable) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            INSERT INTO installed_models (
                id, model_id, install_path, installed_at, status, port,
                process_id, last_used, usage_count, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&installed_model.id)
        .bind(&installed_model.model_id)
        .bind(&installed_model.install_path)
        .bind(installed_model.installed_at)
        .bind(&installed_model.status)
        .bind(installed_model.port)
        .bind(installed_model.process_id)
        .bind(installed_model.last_used)
        .bind(installed_model.usage_count)
        .bind(installed_model.created_at)
        .bind(installed_model.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_installed_model(&self, installed_model: &InstalledModelsTable) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            UPDATE installed_models SET
                install_path = ?, status = ?, port = ?, process_id = ?,
                last_used = ?, usage_count = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&installed_model.install_path)
        .bind(&installed_model.status)
        .bind(installed_model.port)
        .bind(installed_model.process_id)
        .bind(installed_model.last_used)
        .bind(installed_model.usage_count)
        .bind(installed_model.updated_at)
        .bind(&installed_model.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn uninstall_model(&self, model_id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM installed_models WHERE model_id = ?")
            .bind(model_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_installed_models_by_status(&self, status: &str) -> Result<Vec<InstalledModelsTable>, Self::Error> {
        let installed_models = sqlx::query_as::<_, InstalledModelsTable>(
            "SELECT * FROM installed_models WHERE status = ? ORDER BY installed_at DESC"
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

        Ok(installed_models)
    }

    async fn update_model_usage(&self, model_id: Uuid) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            UPDATE installed_models
            SET last_used = ?, usage_count = usage_count + 1, updated_at = ?
            WHERE model_id = ?
            "#
        )
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now())
        .bind(model_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_models_with_install_info(&self) -> Result<Vec<(ModelsTable, Option<InstalledModelsTable>)>, Self::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                m.*,
                im.id as im_id, im.model_id as im_model_id, im.install_path,
                im.installed_at, im.status, im.port, im.process_id,
                im.last_used, im.usage_count, im.created_at as im_created_at,
                im.updated_at as im_updated_at
            FROM models m
            LEFT JOIN installed_models im ON m.id = im.model_id
            ORDER BY m.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            let model = ModelsTable {
                id: row.get("id"),
                name: row.get("name"),
                display_name: row.get("display_name"),
                description: row.get("description"),
                version: row.get("version"),
                model_type: row.get("model_type"),
                size_category: row.get("size_category"),
                file_size: row.get("file_size"),
                provider: row.get("provider"),
                license: row.get("license"),
                tags: row.get("tags"),
                languages: row.get("languages"),
                file_path: row.get("file_path"),
                checksum: row.get("checksum"),
                download_url: row.get("download_url"),
                config: row.get("config"),
                rating: row.get("rating"),
                download_count: row.get("download_count"),
                is_official: row.get("is_official"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            let installed_model = if let Ok(im_id) = row.try_get::<Option<Uuid>, _>("im_id") {
                im_id.map(|_| InstalledModelsTable {
                    id: row.get("im_id"),
                    model_id: row.get("im_model_id"),
                    install_path: row.get("install_path"),
                    installed_at: row.get("installed_at"),
                    status: row.get("status"),
                    port: row.get("port"),
                    process_id: row.get("process_id"),
                    last_used: row.get("last_used"),
                    usage_count: row.get("usage_count"),
                    created_at: row.get("im_created_at"),
                    updated_at: row.get("im_updated_at"),
                })
            } else {
                None
            };

            result.push((model, installed_model));
        }

        Ok(result)
    }
}