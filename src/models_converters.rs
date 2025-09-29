use crate::models_table::{ModelsTable, InstalledModelsTable};
use burncloud_service_models::{Model, InstalledModel, ModelType, ModelSize, ModelStatus};
use serde_json;
use std::collections::HashMap;

/// 将 service Model 转换为数据库 ModelsTable
pub fn service_model_to_db(service_model: &Model) -> ModelsTable {
    ModelsTable {
        id: service_model.id,
        name: service_model.name.clone(),
        display_name: service_model.display_name.clone(),
        description: service_model.description.clone(),
        version: service_model.version.clone(),
        model_type: model_type_to_string(&service_model.model_type),
        size_category: model_size_to_string(&service_model.size_category),
        file_size: service_model.file_size as i64,
        provider: service_model.provider.clone(),
        license: service_model.license.clone(),
        tags: serde_json::to_string(&service_model.tags).unwrap_or_else(|_| "[]".to_string()),
        languages: serde_json::to_string(&service_model.languages).unwrap_or_else(|_| "[]".to_string()),
        file_path: service_model.file_path.clone(),
        checksum: service_model.checksum.clone(),
        download_url: service_model.download_url.clone(),
        config: serde_json::to_string(&service_model.config).unwrap_or_else(|_| "{}".to_string()),
        rating: service_model.rating,
        download_count: service_model.download_count as i64,
        is_official: service_model.is_official,
        created_at: service_model.created_at,
        updated_at: service_model.updated_at,
    }
}

/// 将数据库 ModelsTable 转换为 service Model
pub fn db_model_to_service(db_model: &ModelsTable) -> Result<Model, ConversionError> {
    let tags: Vec<String> = serde_json::from_str(&db_model.tags)
        .map_err(|e| ConversionError::JsonParseError(format!("Failed to parse tags: {}", e)))?;

    let languages: Vec<String> = serde_json::from_str(&db_model.languages)
        .map_err(|e| ConversionError::JsonParseError(format!("Failed to parse languages: {}", e)))?;

    let config: HashMap<String, serde_json::Value> = serde_json::from_str(&db_model.config)
        .map_err(|e| ConversionError::JsonParseError(format!("Failed to parse config: {}", e)))?;

    let model_type = string_to_model_type(&db_model.model_type)?;
    let size_category = string_to_model_size(&db_model.size_category)?;

    Ok(Model {
        id: db_model.id,
        name: db_model.name.clone(),
        display_name: db_model.display_name.clone(),
        description: db_model.description.clone(),
        version: db_model.version.clone(),
        model_type,
        size_category,
        file_size: db_model.file_size as u64,
        provider: db_model.provider.clone(),
        license: db_model.license.clone(),
        tags,
        languages,
        created_at: db_model.created_at,
        updated_at: db_model.updated_at,
        file_path: db_model.file_path.clone(),
        checksum: db_model.checksum.clone(),
        download_url: db_model.download_url.clone(),
        config,
        rating: db_model.rating,
        download_count: db_model.download_count as u64,
        is_official: db_model.is_official,
    })
}

/// 将 service InstalledModel 转换为数据库 InstalledModelsTable
pub fn service_installed_model_to_db(service_installed: &InstalledModel) -> (ModelsTable, InstalledModelsTable) {
    let db_model = service_model_to_db(&service_installed.model);

    let db_installed = InstalledModelsTable {
        id: uuid::Uuid::new_v4(), // 生成新的安装记录ID
        model_id: service_installed.model.id,
        install_path: service_installed.install_path.clone(),
        installed_at: service_installed.installed_at,
        status: model_status_to_string(&service_installed.status),
        port: service_installed.port.map(|p| p as i32),
        process_id: service_installed.process_id.map(|p| p as i32),
        last_used: service_installed.last_used,
        usage_count: service_installed.usage_count as i64,
        created_at: service_installed.installed_at,
        updated_at: service_installed.installed_at,
    };

    (db_model, db_installed)
}

/// 将数据库记录转换为 service InstalledModel
pub fn db_installed_model_to_service(
    db_model: &ModelsTable,
    db_installed: &InstalledModelsTable,
) -> Result<InstalledModel, ConversionError> {
    let service_model = db_model_to_service(db_model)?;
    let status = string_to_model_status(&db_installed.status)?;

    Ok(InstalledModel {
        model: service_model,
        install_path: db_installed.install_path.clone(),
        installed_at: db_installed.installed_at,
        status,
        port: db_installed.port.map(|p| p as u16),
        process_id: db_installed.process_id.map(|p| p as u32),
        last_used: db_installed.last_used,
        usage_count: db_installed.usage_count as u64,
    })
}

/// ModelType 转换函数
fn model_type_to_string(model_type: &ModelType) -> String {
    match model_type {
        ModelType::Chat => "Chat".to_string(),
        ModelType::Code => "Code".to_string(),
        ModelType::Text => "Text".to_string(),
        ModelType::Embedding => "Embedding".to_string(),
        ModelType::Multimodal => "Multimodal".to_string(),
        ModelType::ImageGeneration => "ImageGeneration".to_string(),
        ModelType::Speech => "Speech".to_string(),
    }
}

fn string_to_model_type(s: &str) -> Result<ModelType, ConversionError> {
    match s {
        "Chat" => Ok(ModelType::Chat),
        "Code" => Ok(ModelType::Code),
        "Text" => Ok(ModelType::Text),
        "Embedding" => Ok(ModelType::Embedding),
        "Multimodal" => Ok(ModelType::Multimodal),
        "ImageGeneration" => Ok(ModelType::ImageGeneration),
        "Speech" => Ok(ModelType::Speech),
        _ => Err(ConversionError::InvalidModelType(s.to_string())),
    }
}

/// ModelSize 转换函数
fn model_size_to_string(model_size: &ModelSize) -> String {
    match model_size {
        ModelSize::Small => "Small".to_string(),
        ModelSize::Medium => "Medium".to_string(),
        ModelSize::Large => "Large".to_string(),
        ModelSize::XLarge => "XLarge".to_string(),
    }
}

fn string_to_model_size(s: &str) -> Result<ModelSize, ConversionError> {
    match s {
        "Small" => Ok(ModelSize::Small),
        "Medium" => Ok(ModelSize::Medium),
        "Large" => Ok(ModelSize::Large),
        "XLarge" => Ok(ModelSize::XLarge),
        _ => Err(ConversionError::InvalidModelSize(s.to_string())),
    }
}

/// ModelStatus 转换函数
fn model_status_to_string(model_status: &ModelStatus) -> String {
    match model_status {
        ModelStatus::Running => "Running".to_string(),
        ModelStatus::Stopped => "Stopped".to_string(),
        ModelStatus::Starting => "Starting".to_string(),
        ModelStatus::Stopping => "Stopping".to_string(),
        ModelStatus::Error => "Error".to_string(),
        ModelStatus::Downloading => "Downloading".to_string(),
        ModelStatus::Installing => "Installing".to_string(),
    }
}

fn string_to_model_status(s: &str) -> Result<ModelStatus, ConversionError> {
    match s {
        "Running" => Ok(ModelStatus::Running),
        "Stopped" => Ok(ModelStatus::Stopped),
        "Starting" => Ok(ModelStatus::Starting),
        "Stopping" => Ok(ModelStatus::Stopping),
        "Error" => Ok(ModelStatus::Error),
        "Downloading" => Ok(ModelStatus::Downloading),
        "Installing" => Ok(ModelStatus::Installing),
        _ => Err(ConversionError::InvalidModelStatus(s.to_string())),
    }
}

/// 转换错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("JSON parse error: {0}")]
    JsonParseError(String),

    #[error("Invalid model type: {0}")]
    InvalidModelType(String),

    #[error("Invalid model size: {0}")]
    InvalidModelSize(String),

    #[error("Invalid model status: {0}")]
    InvalidModelStatus(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use burncloud_service_models::{Model, ModelType, ModelSize};

    #[test]
    fn test_service_model_to_db_conversion() {
        let service_model = Model::new(
            "test-model".to_string(),
            "Test Model".to_string(),
            "1.0.0".to_string(),
            ModelType::Chat,
            "Test Provider".to_string(),
            1024 * 1024 * 1024, // 1GB
        );

        let db_model = service_model_to_db(&service_model);

        assert_eq!(db_model.name, "test-model");
        assert_eq!(db_model.display_name, "Test Model");
        assert_eq!(db_model.model_type, "Chat");
        assert_eq!(db_model.size_category, "Small");
        assert_eq!(db_model.file_size, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_db_model_to_service_conversion() {
        let mut db_model = ModelsTable::new(
            "test-model".to_string(),
            "Test Model".to_string(),
            "1.0.0".to_string(),
            "Chat".to_string(),
            "Test Provider".to_string(),
            1024 * 1024 * 1024,
        );
        db_model.tags = r#"["ai", "chat"]"#.to_string();
        db_model.languages = r#"["en", "zh"]"#.to_string();
        db_model.config = r#"{"temperature": 0.7}"#.to_string();

        let service_model = db_model_to_service(&db_model).unwrap();

        assert_eq!(service_model.name, "test-model");
        assert_eq!(service_model.display_name, "Test Model");
        assert_eq!(service_model.model_type, ModelType::Chat);
        assert_eq!(service_model.size_category, ModelSize::Small);
        assert_eq!(service_model.tags, vec!["ai", "chat"]);
        assert_eq!(service_model.languages, vec!["en", "zh"]);
    }

    #[test]
    fn test_model_type_conversions() {
        assert_eq!(model_type_to_string(&ModelType::Chat), "Chat");
        assert_eq!(string_to_model_type("Chat").unwrap(), ModelType::Chat);

        assert!(string_to_model_type("InvalidType").is_err());
    }

    #[test]
    fn test_model_size_conversions() {
        assert_eq!(model_size_to_string(&ModelSize::Large), "Large");
        assert_eq!(string_to_model_size("Large").unwrap(), ModelSize::Large);

        assert!(string_to_model_size("InvalidSize").is_err());
    }
}