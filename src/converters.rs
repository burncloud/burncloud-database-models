use burncloud_service_models as service;
use crate::models::*;
use uuid::Uuid;

/// 转换器：Service Models <-> Database Models

/// 将 Service Model 转换为 Database Model
impl From<service::Model> for DbModel {
    fn from(model: service::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            display_name: model.display_name,
            description: model.description,
            version: model.version,
            model_type: serde_json::to_string(&model.model_type).unwrap_or_default(),
            size_category: serde_json::to_string(&model.size_category).unwrap_or_default(),
            file_size: model.file_size as i64,
            provider: model.provider,
            license: model.license,
            tags: sqlx::types::Json(model.tags),
            languages: sqlx::types::Json(model.languages),
            created_at: model.created_at,
            updated_at: model.updated_at,
            file_path: model.file_path,
            checksum: model.checksum,
            download_url: model.download_url,
            config: sqlx::types::Json(model.config),
            rating: model.rating,
            download_count: model.download_count as i64,
            is_official: model.is_official,
        }
    }
}

/// 将 Database Model 转换为 Service Model
impl TryFrom<DbModel> for service::Model {
    type Error = serde_json::Error;

    fn try_from(db_model: DbModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: db_model.id,
            name: db_model.name,
            display_name: db_model.display_name,
            description: db_model.description,
            version: db_model.version,
            model_type: serde_json::from_str(&db_model.model_type)?,
            size_category: serde_json::from_str(&db_model.size_category)?,
            file_size: db_model.file_size as u64,
            provider: db_model.provider,
            license: db_model.license,
            tags: db_model.tags.0,
            languages: db_model.languages.0,
            created_at: db_model.created_at,
            updated_at: db_model.updated_at,
            file_path: db_model.file_path,
            checksum: db_model.checksum,
            download_url: db_model.download_url,
            config: db_model.config.0,
            rating: db_model.rating,
            download_count: db_model.download_count as u64,
            is_official: db_model.is_official,
        })
    }
}

/// 转换函数 - 避免孤儿规则问题
pub fn convert_installed_model_to_db(installed: service::InstalledModel) -> (DbModel, DbInstalledModel) {
    let db_model = DbModel::from(installed.model);
    let db_installed = DbInstalledModel {
        id: Uuid::new_v4(),
        model_id: db_model.id,
        install_path: installed.install_path,
        installed_at: installed.installed_at,
        status: serde_json::to_string(&installed.status).unwrap_or_default(),
        port: installed.port.map(|p| p as i32),
        process_id: installed.process_id.map(|p| p as i32),
        last_used: installed.last_used,
        usage_count: installed.usage_count as i64,
    };
    (db_model, db_installed)
}

pub fn convert_db_to_installed_model(
    db_model: DbModel,
    db_installed: DbInstalledModel,
) -> Result<service::InstalledModel, serde_json::Error> {
    let model = service::Model::try_from(db_model)?;
    Ok(service::InstalledModel {
        model,
        install_path: db_installed.install_path,
        installed_at: db_installed.installed_at,
        status: serde_json::from_str(&db_installed.status)?,
        port: db_installed.port.map(|p| p as u16),
        process_id: db_installed.process_id.map(|p| p as u32),
        last_used: db_installed.last_used,
        usage_count: db_installed.usage_count as u64,
    })
}

pub fn convert_available_model_to_db(available: service::AvailableModel) -> (DbModel, DbAvailableModel) {
    let db_model = DbModel::from(available.model);
    let db_available = DbAvailableModel {
        id: Uuid::new_v4(),
        model_id: db_model.id,
        is_installed: available.is_installed,
        published_at: available.published_at,
        last_updated: available.last_updated,
        system_requirements: sqlx::types::Json(DbSystemRequirements {
            min_memory_gb: available.system_requirements.min_memory_gb,
            recommended_memory_gb: available.system_requirements.recommended_memory_gb,
            min_disk_space_gb: available.system_requirements.min_disk_space_gb,
            requires_gpu: available.system_requirements.requires_gpu,
            supported_os: available.system_requirements.supported_os,
            supported_architectures: available.system_requirements.supported_architectures,
        }),
    };
    (db_model, db_available)
}

pub fn convert_db_to_available_model(
    db_model: DbModel,
    db_available: DbAvailableModel,
) -> Result<service::AvailableModel, serde_json::Error> {
    let model = service::Model::try_from(db_model)?;
    Ok(service::AvailableModel {
        model,
        is_installed: db_available.is_installed,
        published_at: db_available.published_at,
        last_updated: db_available.last_updated,
        system_requirements: service::SystemRequirements {
            min_memory_gb: db_available.system_requirements.0.min_memory_gb,
            recommended_memory_gb: db_available.system_requirements.0.recommended_memory_gb,
            min_disk_space_gb: db_available.system_requirements.0.min_disk_space_gb,
            requires_gpu: db_available.system_requirements.0.requires_gpu,
            supported_os: db_available.system_requirements.0.supported_os,
            supported_architectures: db_available.system_requirements.0.supported_architectures,
        },
    })
}

impl From<service::RuntimeConfig> for DbRuntimeConfig {
    fn from(config: service::RuntimeConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: format!("runtime_config_{}", Uuid::new_v4()),
            max_context_length: config.max_context_length.map(|c| c as i32),
            temperature: config.temperature,
            top_p: config.top_p,
            top_k: config.top_k.map(|k| k as i32),
            max_tokens: config.max_tokens.map(|t| t as i32),
            stop_sequences: sqlx::types::Json(config.stop_sequences),
            batch_size: config.batch_size.map(|b| b as i32),
            max_concurrent_requests: config.max_concurrent_requests.map(|r| r as i32),
            gpu_device_ids: sqlx::types::Json(config.gpu_device_ids.into_iter().map(|id| id as i32).collect()),
            memory_limit_mb: config.memory_limit_mb.map(|m| m as i64),
            enable_streaming: config.enable_streaming,
            custom_params: sqlx::types::Json(config.custom_params),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl From<DbRuntimeConfig> for service::RuntimeConfig {
    fn from(db_config: DbRuntimeConfig) -> Self {
        Self {
            max_context_length: db_config.max_context_length.map(|c| c as u32),
            temperature: db_config.temperature,
            top_p: db_config.top_p,
            top_k: db_config.top_k.map(|k| k as u32),
            max_tokens: db_config.max_tokens.map(|t| t as u32),
            stop_sequences: db_config.stop_sequences.0,
            batch_size: db_config.batch_size.map(|b| b as u32),
            max_concurrent_requests: db_config.max_concurrent_requests.map(|r| r as u32),
            gpu_device_ids: db_config.gpu_device_ids.0.into_iter().map(|id| id as u32).collect(),
            memory_limit_mb: db_config.memory_limit_mb.map(|m| m as u64),
            enable_streaming: db_config.enable_streaming,
            custom_params: db_config.custom_params.0,
        }
    }
}