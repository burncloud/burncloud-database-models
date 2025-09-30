use crate::models_table::{ModelsTable, InstalledModelsTable, CREATE_MODELS_TABLE_SQL, CREATE_INSTALLED_MODELS_TABLE_SQL};
use burncloud_database_core::{Database, DatabaseError};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

/// Repository for managing models in the database
///
/// This repository provides CRUD operations for both models and installed models,
/// along with table management and complex queries.
pub struct ModelsRepository {
    database: Arc<Database>,
}

impl ModelsRepository {
    /// Create a new ModelsRepository instance
    pub async fn new(database: Arc<Database>) -> Result<Self, DatabaseError> {
        Ok(Self { database })
    }

    /// Ensure that the required database tables exist
    pub async fn ensure_tables_exist(&self) -> Result<(), DatabaseError> {
        // Create models table
        self.database.execute_query(CREATE_MODELS_TABLE_SQL).await?;

        // Create installed_models table
        self.database.execute_query(CREATE_INSTALLED_MODELS_TABLE_SQL).await?;

        Ok(())
    }

    // === Models table operations ===

    /// Create a new model in the database
    pub async fn create_model(&self, model: &ModelsTable) -> Result<ModelsTable, DatabaseError> {
        let query = r#"
            INSERT INTO models (
                id, name, display_name, description, version, model_type,
                size_category, file_size, provider, license, tags, languages,
                file_path, checksum, download_url, config, rating,
                download_count, is_official, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
        "#;

        let params = vec![
            model.id.to_string(),
            model.name.clone(),
            model.display_name.clone(),
            model.description.clone().unwrap_or_default(),
            model.version.clone(),
            model.model_type.clone(),
            model.size_category.clone(),
            model.file_size.to_string(),
            model.provider.clone(),
            model.license.clone().unwrap_or_default(),
            model.tags.clone(),
            model.languages.clone(),
            model.file_path.clone().unwrap_or_default(),
            model.checksum.clone().unwrap_or_default(),
            model.download_url.clone().unwrap_or_default(),
            model.config.clone(),
            model.rating.map(|r| r.to_string()).unwrap_or_default(),
            model.download_count.to_string(),
            model.is_official.to_string(),
            model.created_at.to_rfc3339(),
            model.updated_at.to_rfc3339(),
        ];

        self.database.execute_query_with_params(query, params).await?;

        // Return the created model
        Ok(model.clone())
    }

    /// Get a model by its ID
    pub async fn get_model_by_id(&self, id: Uuid) -> Result<Option<ModelsTable>, DatabaseError> {
        let query = "SELECT * FROM models WHERE id = $1";
        let params = vec![id.to_string()];

        let rows = self.database.query_with_params(query, params).await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let row = &rows[0];
        Ok(Some(self.row_to_models_table(row)?))
    }

    /// Get a model by its name
    pub async fn get_model_by_name(&self, name: &str) -> Result<Option<ModelsTable>, DatabaseError> {
        let query = "SELECT * FROM models WHERE name = $1";
        let params = vec![name.to_string()];

        let rows = self.database.query_with_params(query, params).await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let row = &rows[0];
        Ok(Some(self.row_to_models_table(row)?))
    }

    /// Get all models from the database
    pub async fn get_all_models(&self) -> Result<Vec<ModelsTable>, DatabaseError> {
        let query = "SELECT * FROM models ORDER BY created_at DESC";
        let rows = self.database.query(query).await?;

        let mut models = Vec::new();
        for row in rows {
            models.push(self.row_to_models_table(&row)?);
        }

        Ok(models)
    }

    /// Update an existing model
    pub async fn update_model(&self, model: &ModelsTable) -> Result<ModelsTable, DatabaseError> {
        let query = r#"
            UPDATE models SET
                display_name = $2, description = $3, version = $4, model_type = $5,
                size_category = $6, file_size = $7, provider = $8, license = $9,
                tags = $10, languages = $11, file_path = $12, checksum = $13,
                download_url = $14, config = $15, rating = $16, download_count = $17,
                is_official = $18, updated_at = $19
            WHERE id = $1
        "#;

        let params = vec![
            model.id.to_string(),
            model.display_name.clone(),
            model.description.clone().unwrap_or_default(),
            model.version.clone(),
            model.model_type.clone(),
            model.size_category.clone(),
            model.file_size.to_string(),
            model.provider.clone(),
            model.license.clone().unwrap_or_default(),
            model.tags.clone(),
            model.languages.clone(),
            model.file_path.clone().unwrap_or_default(),
            model.checksum.clone().unwrap_or_default(),
            model.download_url.clone().unwrap_or_default(),
            model.config.clone(),
            model.rating.map(|r| r.to_string()).unwrap_or_default(),
            model.download_count.to_string(),
            model.is_official.to_string(),
            model.updated_at.to_rfc3339(),
        ];

        self.database.execute_query_with_params(query, params).await?;

        // Return the updated model
        Ok(model.clone())
    }

    /// Delete a model by its ID
    pub async fn delete_model(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let query = "DELETE FROM models WHERE id = $1";
        let params = vec![id.to_string()];

        let result = self.database.execute_query_with_params(query, params).await?;

        // Check the number of rows affected
        Ok(result.rows_affected() > 0)
    }

    // === Installed models operations ===

    /// Get all installed models with their associated model data
    pub async fn get_installed_models(&self) -> Result<Vec<(ModelsTable, InstalledModelsTable)>, DatabaseError> {
        let query = r#"
            SELECT
                m.*,
                im.id as im_id, im.model_id, im.install_path, im.installed_at,
                im.status, im.port, im.process_id, im.last_used, im.usage_count,
                im.created_at as im_created_at, im.updated_at as im_updated_at
            FROM models m
            INNER JOIN installed_models im ON m.id = im.model_id
            ORDER BY im.installed_at DESC
        "#;

        let rows = self.database.query(query).await?;
        let mut result = Vec::new();

        for row in rows {
            let model = self.row_to_models_table(&row)?;
            let installed = self.row_to_installed_models_table(&row, "im_")?;
            result.push((model, installed));
        }

        Ok(result)
    }

    /// Install a model
    pub async fn install_model(&self, model_id: Uuid, install_path: String) -> Result<InstalledModelsTable, DatabaseError> {
        let installed_model = InstalledModelsTable::new(model_id, install_path);

        let query = r#"
            INSERT INTO installed_models (
                id, model_id, install_path, installed_at, status, port,
                process_id, last_used, usage_count, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#;

        let params = vec![
            installed_model.id.to_string(),
            installed_model.model_id.to_string(),
            installed_model.install_path.clone(),
            installed_model.installed_at.to_rfc3339(),
            installed_model.status.clone(),
            installed_model.port.map(|p| p.to_string()).unwrap_or_default(),
            installed_model.process_id.map(|p| p.to_string()).unwrap_or_default(),
            installed_model.last_used.map(|t| t.to_rfc3339()).unwrap_or_default(),
            installed_model.usage_count.to_string(),
            installed_model.created_at.to_rfc3339(),
            installed_model.updated_at.to_rfc3339(),
        ];

        self.database.execute_query_with_params(query, params).await?;

        Ok(installed_model)
    }

    /// Update model status
    pub async fn update_model_status(&self, model_id: Uuid, status: String) -> Result<(), DatabaseError> {
        let query = r#"
            UPDATE installed_models
            SET status = $2, updated_at = $3
            WHERE model_id = $1
        "#;

        let params = vec![
            model_id.to_string(),
            status,
            Utc::now().to_rfc3339(),
        ];

        self.database.execute_query_with_params(query, params).await?;
        Ok(())
    }

    // === Search and filtering operations ===

    /// Search models by name, display name, or description
    pub async fn search_models(&self, query: &str, limit: Option<u32>) -> Result<Vec<ModelsTable>, DatabaseError> {
        let search_query = r#"
            SELECT * FROM models
            WHERE name LIKE $1 OR display_name LIKE $1 OR description LIKE $1
            ORDER BY created_at DESC
            LIMIT $2
        "#;

        let search_pattern = format!("%{}%", query);
        let limit_val = limit.unwrap_or(50).to_string();
        let params = vec![search_pattern, limit_val];

        let rows = self.database.query_with_params(search_query, params).await?;

        let mut models = Vec::new();
        for row in rows {
            models.push(self.row_to_models_table(&row)?);
        }

        Ok(models)
    }

    /// Get models by type
    pub async fn get_models_by_type(&self, model_type: &str) -> Result<Vec<ModelsTable>, DatabaseError> {
        let query = "SELECT * FROM models WHERE model_type = $1 ORDER BY created_at DESC";
        let params = vec![model_type.to_string()];

        let rows = self.database.query_with_params(query, params).await?;

        let mut models = Vec::new();
        for row in rows {
            models.push(self.row_to_models_table(&row)?);
        }

        Ok(models)
    }

    /// Get models by provider
    pub async fn get_models_by_provider(&self, provider: &str) -> Result<Vec<ModelsTable>, DatabaseError> {
        let query = "SELECT * FROM models WHERE provider = $1 ORDER BY created_at DESC";
        let params = vec![provider.to_string()];

        let rows = self.database.query_with_params(query, params).await?;

        let mut models = Vec::new();
        for row in rows {
            models.push(self.row_to_models_table(&row)?);
        }

        Ok(models)
    }

    /// Get official models
    pub async fn get_official_models(&self) -> Result<Vec<ModelsTable>, DatabaseError> {
        let query = "SELECT * FROM models WHERE is_official = 'true' ORDER BY created_at DESC";

        let rows = self.database.query(query).await?;

        let mut models = Vec::new();
        for row in rows {
            models.push(self.row_to_models_table(&row)?);
        }

        Ok(models)
    }

    // === Utility methods ===

    /// Convert a database row to ModelsTable
    fn row_to_models_table(&self, row: &sqlx::sqlite::SqliteRow) -> Result<ModelsTable, DatabaseError> {
        use sqlx::Row;

        let id: String = row.try_get("id")
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid id: {}", e) })?;
        let id = Uuid::parse_str(&id)
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid UUID format for id: {}", e) })?;

        let file_size: i64 = row.try_get("file_size")
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid file_size: {}", e) })?;

        let download_count: i64 = row.try_get("download_count").unwrap_or(0);

        let rating_str: Option<String> = row.try_get("rating").ok();
        let rating = rating_str
            .and_then(|s| if s.is_empty() { None } else { s.parse::<f32>().ok() });

        let is_official_str: String = row.try_get("is_official").unwrap_or_else(|_| "false".to_string());
        let is_official = is_official_str == "true" || is_official_str == "1";

        let created_at_str: String = row.try_get("created_at")
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid created_at: {}", e) })?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid created_at format: {}", e) })?;

        let updated_at_str: String = row.try_get("updated_at")
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid updated_at: {}", e) })?;
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid updated_at format: {}", e) })?;

        Ok(ModelsTable {
            id,
            name: row.try_get("name").unwrap_or_default(),
            display_name: row.try_get("display_name").unwrap_or_default(),
            description: {
                let desc: Option<String> = row.try_get("description").ok();
                desc.filter(|s| !s.is_empty())
            },
            version: row.try_get("version").unwrap_or_default(),
            model_type: row.try_get("model_type").unwrap_or_default(),
            size_category: row.try_get("size_category").unwrap_or_default(),
            file_size,
            provider: row.try_get("provider").unwrap_or_default(),
            license: {
                let license: Option<String> = row.try_get("license").ok();
                license.filter(|s| !s.is_empty())
            },
            tags: row.try_get("tags").unwrap_or_else(|_| "[]".to_string()),
            languages: row.try_get("languages").unwrap_or_else(|_| "[]".to_string()),
            file_path: {
                let path: Option<String> = row.try_get("file_path").ok();
                path.filter(|s| !s.is_empty())
            },
            checksum: {
                let checksum: Option<String> = row.try_get("checksum").ok();
                checksum.filter(|s| !s.is_empty())
            },
            download_url: {
                let url: Option<String> = row.try_get("download_url").ok();
                url.filter(|s| !s.is_empty())
            },
            config: row.try_get("config").unwrap_or_else(|_| "{}".to_string()),
            rating,
            download_count,
            is_official,
            created_at,
            updated_at,
        })
    }

    /// Convert a database row to InstalledModelsTable
    fn row_to_installed_models_table(&self, row: &sqlx::sqlite::SqliteRow, prefix: &str) -> Result<InstalledModelsTable, DatabaseError> {
        use sqlx::Row;

        let id_key = if prefix.is_empty() { "id".to_string() } else { format!("{}id", prefix) };
        let id_str: String = row.try_get(id_key.as_str())
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid or missing installed model id: {}", e) })?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid UUID format for installed model id: {}", e) })?;

        let model_id_str: String = row.try_get("model_id")
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid model_id: {}", e) })?;
        let model_id = Uuid::parse_str(&model_id_str)
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid UUID format for model_id: {}", e) })?;

        let installed_at_key = if prefix.is_empty() { "installed_at".to_string() } else { format!("{}installed_at", prefix) };
        let installed_at_str: String = row.try_get(installed_at_key.as_str())
            .or_else(|_| row.try_get("installed_at"))
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid installed_at: {}", e) })?;
        let installed_at = chrono::DateTime::parse_from_rfc3339(&installed_at_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| DatabaseError::InvalidData { message: format!("Invalid installed_at format: {}", e) })?;

        let created_at_key = if prefix.is_empty() { "created_at".to_string() } else { format!("{}created_at", prefix) };
        let created_at = if let Ok(created_at_str) = row.try_get::<String, _>(created_at_key.as_str()) {
            chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(installed_at)
        } else {
            installed_at
        };

        let updated_at_key = if prefix.is_empty() { "updated_at".to_string() } else { format!("{}updated_at", prefix) };
        let updated_at = if let Ok(updated_at_str) = row.try_get::<String, _>(updated_at_key.as_str()) {
            chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(installed_at)
        } else {
            installed_at
        };

        let port: Option<i32> = row.try_get("port").ok()
            .and_then(|s: String| if s.is_empty() { None } else { s.parse().ok() });

        let process_id: Option<i32> = row.try_get("process_id").ok()
            .and_then(|s: String| if s.is_empty() { None } else { s.parse().ok() });

        let last_used = row.try_get::<String, _>("last_used").ok()
            .and_then(|s| if s.is_empty() { None } else {
                chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))
            });

        let usage_count: i64 = row.try_get("usage_count").ok()
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(0);

        Ok(InstalledModelsTable {
            id,
            model_id,
            install_path: row.try_get("install_path").unwrap_or_default(),
            installed_at,
            status: row.try_get("status").unwrap_or_else(|_| "Stopped".to_string()),
            port,
            process_id,
            last_used,
            usage_count,
            created_at,
            updated_at,
        })
    }
}