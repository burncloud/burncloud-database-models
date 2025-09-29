use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;

use crate::models_repository::{ModelsRepository, SqliteModelsRepository};
use crate::models_converters::{
    service_model_to_db, db_model_to_service, service_installed_model_to_db,
    db_installed_model_to_service, ConversionError,
};
use burncloud_service_models::{Model, InstalledModel, ModelType, ModelStatus};
use burncloud_database_client::DatabasePool;

/// Models 表的业务服务层
/// 提供高级的业务逻辑操作，处理类型转换和业务规则
pub struct ModelsService {
    repository: Arc<dyn ModelsRepository<Error = sqlx::Error>>,
}

impl ModelsService {
    /// 从数据库连接池创建服务
    pub fn new_from_pool(pool: sqlx::SqlitePool) -> Self {
        let repository = Arc::new(SqliteModelsRepository::new(pool));
        Self { repository }
    }

    /// 从自定义 repository 创建服务
    pub fn new_with_repository(repository: Arc<dyn ModelsRepository<Error = sqlx::Error>>) -> Self {
        Self { repository }
    }

    // === 模型相关业务操作 ===

    /// 获取所有模型
    pub async fn get_all_models(&self) -> Result<Vec<Model>, ModelsServiceError> {
        let db_models = self.repository.get_all_models().await?;
        let mut service_models = Vec::new();

        for db_model in db_models {
            let service_model = db_model_to_service(&db_model)?;
            service_models.push(service_model);
        }

        Ok(service_models)
    }

    /// 根据ID获取模型
    pub async fn get_model_by_id(&self, id: Uuid) -> Result<Option<Model>, ModelsServiceError> {
        if let Some(db_model) = self.repository.get_model_by_id(id).await? {
            let service_model = db_model_to_service(&db_model)?;
            Ok(Some(service_model))
        } else {
            Ok(None)
        }
    }

    /// 根据名称获取模型
    pub async fn get_model_by_name(&self, name: &str) -> Result<Option<Model>, ModelsServiceError> {
        if let Some(db_model) = self.repository.get_model_by_name(name).await? {
            let service_model = db_model_to_service(&db_model)?;
            Ok(Some(service_model))
        } else {
            Ok(None)
        }
    }

    /// 创建新模型
    pub async fn create_model(&self, model: &Model) -> Result<(), ModelsServiceError> {
        // 检查模型名称是否已存在
        if self.repository.get_model_by_name(&model.name).await?.is_some() {
            return Err(ModelsServiceError::ModelAlreadyExists(model.name.clone()));
        }

        let db_model = service_model_to_db(model);
        self.repository.create_model(&db_model).await?;
        Ok(())
    }

    /// 更新模型
    pub async fn update_model(&self, model: &Model) -> Result<(), ModelsServiceError> {
        // 检查模型是否存在
        if self.repository.get_model_by_id(model.id).await?.is_none() {
            return Err(ModelsServiceError::ModelNotFound(model.id));
        }

        let mut db_model = service_model_to_db(model);
        db_model.mark_updated();
        self.repository.update_model(&db_model).await?;
        Ok(())
    }

    /// 删除模型
    pub async fn delete_model(&self, id: Uuid) -> Result<(), ModelsServiceError> {
        // 检查是否有已安装的实例
        if self.repository.get_installed_model_by_model_id(id).await?.is_some() {
            return Err(ModelsServiceError::ModelHasInstalledInstances(id));
        }

        // 检查模型是否存在
        if self.repository.get_model_by_id(id).await?.is_none() {
            return Err(ModelsServiceError::ModelNotFound(id));
        }

        self.repository.delete_model(id).await?;
        Ok(())
    }

    /// 搜索模型
    pub async fn search_models(&self, query: &str, limit: Option<i64>) -> Result<Vec<Model>, ModelsServiceError> {
        let db_models = self.repository.search_models(query, limit).await?;
        let mut service_models = Vec::new();

        for db_model in db_models {
            let service_model = db_model_to_service(&db_model)?;
            service_models.push(service_model);
        }

        Ok(service_models)
    }

    /// 根据类型获取模型
    pub async fn get_models_by_type(&self, model_type: ModelType) -> Result<Vec<Model>, ModelsServiceError> {
        let type_str = match model_type {
            ModelType::Chat => "Chat",
            ModelType::Code => "Code",
            ModelType::Text => "Text",
            ModelType::Embedding => "Embedding",
            ModelType::Multimodal => "Multimodal",
            ModelType::ImageGeneration => "ImageGeneration",
            ModelType::Speech => "Speech",
        };

        let db_models = self.repository.get_models_by_type(type_str).await?;
        let mut service_models = Vec::new();

        for db_model in db_models {
            let service_model = db_model_to_service(&db_model)?;
            service_models.push(service_model);
        }

        Ok(service_models)
    }

    /// 根据提供商获取模型
    pub async fn get_models_by_provider(&self, provider: &str) -> Result<Vec<Model>, ModelsServiceError> {
        let db_models = self.repository.get_models_by_provider(provider).await?;
        let mut service_models = Vec::new();

        for db_model in db_models {
            let service_model = db_model_to_service(&db_model)?;
            service_models.push(service_model);
        }

        Ok(service_models)
    }

    /// 获取官方模型
    pub async fn get_official_models(&self) -> Result<Vec<Model>, ModelsServiceError> {
        let db_models = self.repository.get_official_models().await?;
        let mut service_models = Vec::new();

        for db_model in db_models {
            let service_model = db_model_to_service(&db_model)?;
            service_models.push(service_model);
        }

        Ok(service_models)
    }

    /// 增加模型下载次数
    pub async fn increment_download_count(&self, id: Uuid) -> Result<(), ModelsServiceError> {
        if self.repository.get_model_by_id(id).await?.is_none() {
            return Err(ModelsServiceError::ModelNotFound(id));
        }

        self.repository.increment_download_count(id).await?;
        Ok(())
    }

    // === 已安装模型相关业务操作 ===

    /// 获取所有已安装模型
    pub async fn get_all_installed_models(&self) -> Result<Vec<InstalledModel>, ModelsServiceError> {
        let models_with_install = self.repository.get_models_with_install_info().await?;
        let mut installed_models = Vec::new();

        for (db_model, db_installed_opt) in models_with_install {
            if let Some(db_installed) = db_installed_opt {
                let installed_model = db_installed_model_to_service(&db_model, &db_installed)?;
                installed_models.push(installed_model);
            }
        }

        Ok(installed_models)
    }

    /// 根据模型ID获取已安装模型
    pub async fn get_installed_model_by_model_id(&self, model_id: Uuid) -> Result<Option<InstalledModel>, ModelsServiceError> {
        if let Some(db_installed) = self.repository.get_installed_model_by_model_id(model_id).await? {
            if let Some(db_model) = self.repository.get_model_by_id(model_id).await? {
                let installed_model = db_installed_model_to_service(&db_model, &db_installed)?;
                Ok(Some(installed_model))
            } else {
                Err(ModelsServiceError::ModelNotFound(model_id))
            }
        } else {
            Ok(None)
        }
    }

    /// 安装模型
    pub async fn install_model(&self, installed_model: &InstalledModel) -> Result<(), ModelsServiceError> {
        // 检查模型是否存在
        if self.repository.get_model_by_id(installed_model.model.id).await?.is_none() {
            return Err(ModelsServiceError::ModelNotFound(installed_model.model.id));
        }

        // 检查是否已经安装
        if self.repository.get_installed_model_by_model_id(installed_model.model.id).await?.is_some() {
            return Err(ModelsServiceError::ModelAlreadyInstalled(installed_model.model.id));
        }

        let (_, db_installed) = service_installed_model_to_db(installed_model);
        self.repository.install_model(&db_installed).await?;
        Ok(())
    }

    /// 更新已安装模型
    pub async fn update_installed_model(&self, installed_model: &InstalledModel) -> Result<(), ModelsServiceError> {
        // 检查安装记录是否存在
        if self.repository.get_installed_model_by_model_id(installed_model.model.id).await?.is_none() {
            return Err(ModelsServiceError::ModelNotInstalled(installed_model.model.id));
        }

        let (_, mut db_installed) = service_installed_model_to_db(installed_model);
        db_installed.updated_at = chrono::Utc::now();
        self.repository.update_installed_model(&db_installed).await?;
        Ok(())
    }

    /// 卸载模型
    pub async fn uninstall_model(&self, model_id: Uuid) -> Result<(), ModelsServiceError> {
        // 检查安装记录是否存在
        if self.repository.get_installed_model_by_model_id(model_id).await?.is_none() {
            return Err(ModelsServiceError::ModelNotInstalled(model_id));
        }

        self.repository.uninstall_model(model_id).await?;
        Ok(())
    }

    /// 根据状态获取已安装模型
    pub async fn get_installed_models_by_status(&self, status: ModelStatus) -> Result<Vec<InstalledModel>, ModelsServiceError> {
        let status_str = match status {
            ModelStatus::Running => "Running",
            ModelStatus::Stopped => "Stopped",
            ModelStatus::Starting => "Starting",
            ModelStatus::Stopping => "Stopping",
            ModelStatus::Error => "Error",
            ModelStatus::Downloading => "Downloading",
            ModelStatus::Installing => "Installing",
        };

        let db_installed_models = self.repository.get_installed_models_by_status(status_str).await?;
        let mut installed_models = Vec::new();

        for db_installed in db_installed_models {
            if let Some(db_model) = self.repository.get_model_by_id(db_installed.model_id).await? {
                let installed_model = db_installed_model_to_service(&db_model, &db_installed)?;
                installed_models.push(installed_model);
            }
        }

        Ok(installed_models)
    }

    /// 更新模型使用情况
    pub async fn update_model_usage(&self, model_id: Uuid) -> Result<(), ModelsServiceError> {
        // 检查安装记录是否存在
        if self.repository.get_installed_model_by_model_id(model_id).await?.is_none() {
            return Err(ModelsServiceError::ModelNotInstalled(model_id));
        }

        self.repository.update_model_usage(model_id).await?;
        Ok(())
    }

    /// 获取模型和安装信息的联合查询
    pub async fn get_models_with_install_info(&self) -> Result<Vec<(Model, Option<InstalledModel>)>, ModelsServiceError> {
        let models_with_install = self.repository.get_models_with_install_info().await?;
        let mut result = Vec::new();

        for (db_model, db_installed_opt) in models_with_install {
            let service_model = db_model_to_service(&db_model)?;
            let service_installed = if let Some(db_installed) = db_installed_opt {
                Some(db_installed_model_to_service(&db_model, &db_installed)?)
            } else {
                None
            };

            result.push((service_model, service_installed));
        }

        Ok(result)
    }
}

/// Models 服务层错误类型
#[derive(Debug, thiserror::Error)]
pub enum ModelsServiceError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Conversion error: {0}")]
    ConversionError(#[from] ConversionError),

    #[error("Model not found: {0}")]
    ModelNotFound(Uuid),

    #[error("Model already exists: {0}")]
    ModelAlreadyExists(String),

    #[error("Model already installed: {0}")]
    ModelAlreadyInstalled(Uuid),

    #[error("Model not installed: {0}")]
    ModelNotInstalled(Uuid),

    #[error("Model has installed instances and cannot be deleted: {0}")]
    ModelHasInstalledInstances(Uuid),
}

/// 工厂函数：从数据库连接客户端创建 ModelsService
pub async fn create_models_service_from_client(
    db_client: &burncloud_database_client::DatabaseClient,
) -> Result<ModelsService, ModelsServiceError> {
    let pool = db_client.get_sqlite_pool()
        .map_err(|e| ModelsServiceError::DatabaseError(
            sqlx::Error::Configuration(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get SQLite pool: {}", e)
            )))
        ))?;

    Ok(ModelsService::new_from_pool(pool))
}

#[cfg(test)]
mod tests {
    use super::*;
    use burncloud_service_models::{Model, ModelType};
    use tempfile::tempdir;

    async fn create_test_service() -> ModelsService {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.to_string_lossy());

        let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();

        // 创建表
        sqlx::query(crate::models_table::CREATE_MODELS_TABLE_SQL)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(crate::models_table::CREATE_INSTALLED_MODELS_TABLE_SQL)
            .execute(&pool)
            .await
            .unwrap();

        ModelsService::new_from_pool(pool)
    }

    #[tokio::test]
    async fn test_create_and_get_model() {
        let service = create_test_service().await;

        let model = Model::new(
            "test-model".to_string(),
            "Test Model".to_string(),
            "1.0.0".to_string(),
            ModelType::Chat,
            "Test Provider".to_string(),
            1024 * 1024 * 1024, // 1GB
        );

        // 创建模型
        service.create_model(&model).await.unwrap();

        // 获取模型
        let retrieved_model = service.get_model_by_name("test-model").await.unwrap().unwrap();
        assert_eq!(retrieved_model.name, "test-model");
        assert_eq!(retrieved_model.display_name, "Test Model");
    }

    #[tokio::test]
    async fn test_duplicate_model_creation() {
        let service = create_test_service().await;

        let model = Model::new(
            "duplicate-model".to_string(),
            "Duplicate Model".to_string(),
            "1.0.0".to_string(),
            ModelType::Chat,
            "Test Provider".to_string(),
            1024 * 1024 * 1024,
        );

        // 第一次创建应该成功
        service.create_model(&model).await.unwrap();

        // 第二次创建应该失败
        let result = service.create_model(&model).await;
        assert!(matches!(result, Err(ModelsServiceError::ModelAlreadyExists(_))));
    }
}