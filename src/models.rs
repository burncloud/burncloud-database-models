use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 数据库模型表 - 对应 burncloud_service_models::Model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbModel {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub version: String,
    pub model_type: String, // JSON string of ModelType
    pub size_category: String, // JSON string of ModelSize
    pub file_size: i64,
    pub provider: String,
    pub license: Option<String>,
    pub tags: sqlx::types::Json<Vec<String>>,
    pub languages: sqlx::types::Json<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub file_path: Option<String>,
    pub checksum: Option<String>,
    pub download_url: Option<String>,
    pub config: sqlx::types::Json<HashMap<String, serde_json::Value>>,
    pub rating: Option<f32>,
    pub download_count: i64,
    pub is_official: bool,
}

/// 数据库已安装模型表 - 对应 burncloud_service_models::InstalledModel
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbInstalledModel {
    pub id: Uuid,
    pub model_id: Uuid,
    pub install_path: String,
    pub installed_at: DateTime<Utc>,
    pub status: String, // JSON string of ModelStatus
    pub port: Option<i32>,
    pub process_id: Option<i32>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: i64,
}

/// 数据库可用模型表 - 对应 burncloud_service_models::AvailableModel
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbAvailableModel {
    pub id: Uuid,
    pub model_id: Uuid,
    pub is_installed: bool,
    pub published_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub system_requirements: sqlx::types::Json<DbSystemRequirements>,
}

/// 数据库系统要求 - 对应 burncloud_service_models::SystemRequirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSystemRequirements {
    pub min_memory_gb: f32,
    pub recommended_memory_gb: f32,
    pub min_disk_space_gb: f32,
    pub requires_gpu: bool,
    pub supported_os: Vec<String>,
    pub supported_architectures: Vec<String>,
}

/// 数据库运行时配置表 - 对应 burncloud_service_models::RuntimeConfig
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbRuntimeConfig {
    pub id: Uuid,
    pub name: String,
    pub max_context_length: Option<i32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub max_tokens: Option<i32>,
    pub stop_sequences: sqlx::types::Json<Vec<String>>,
    pub batch_size: Option<i32>,
    pub max_concurrent_requests: Option<i32>,
    pub gpu_device_ids: sqlx::types::Json<Vec<i32>>,
    pub memory_limit_mb: Option<i64>,
    pub enable_streaming: bool,
    pub custom_params: sqlx::types::Json<HashMap<String, serde_json::Value>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 数据库模型运行时表 - 对应 burncloud_service_models::ModelRuntime
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbModelRuntime {
    pub id: Uuid,
    pub model_id: Uuid,
    pub runtime_config_id: Uuid,
    pub name: String,
    pub port: i32,
    pub process_id: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub status: String, // JSON string of ModelStatus
    pub health_endpoint: String,
    pub api_endpoint: String,
    pub log_file: Option<String>,
    pub environment: sqlx::types::Json<HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 数据库运行时指标表 - 对应 burncloud_service_models::RuntimeMetrics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbRuntimeMetrics {
    pub id: Uuid,
    pub runtime_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: i64,
    pub gpu_usage_percent: Option<f32>,
    pub gpu_memory_usage_mb: Option<i64>,
    pub active_connections: i32,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub avg_response_time_ms: f32,
    pub throughput_rps: f32,
    pub queue_length: i32,
}

/// 数据库运行时事件表 - 对应 burncloud_service_models::RuntimeEvent
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbRuntimeEvent {
    pub id: Uuid,
    pub runtime_id: Uuid,
    pub event_type: String, // JSON string of RuntimeEventType
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub details: Option<sqlx::types::Json<serde_json::Value>>,
    pub severity: String, // JSON string of EventSeverity
}

/// 数据库模型仓库表 - 对应 burncloud_service_models::ModelRepository
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbModelRepository {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub repo_type: String, // JSON string of RepositoryType
    pub enabled: bool,
    pub auth_config: Option<sqlx::types::Json<DbRepositoryAuth>>,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_status: String, // JSON string of SyncStatus
    pub description: Option<String>,
    pub tags: sqlx::types::Json<Vec<String>>,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 数据库仓库认证信息 - 对应 burncloud_service_models::RepositoryAuth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbRepositoryAuth {
    pub auth_type: String, // JSON string of AuthType
    pub username: Option<String>,
    pub token: Option<String>,
    pub api_key: Option<String>,
    pub extra_params: HashMap<String, String>,
}

/// 数据库仓库索引表 - 对应 burncloud_service_models::RepositoryIndex
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbRepositoryIndex {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub version: String,
    pub updated_at: DateTime<Utc>,
    pub checksum: Option<String>,
    pub metadata: sqlx::types::Json<HashMap<String, serde_json::Value>>,
}

/// 数据库仓库模型表 - 对应 burncloud_service_models::RepositoryModel
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbRepositoryModel {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub model_id: Uuid,
    pub repo_model_id: String,
    pub repo_path: String,
    pub download_urls: sqlx::types::Json<Vec<DbDownloadUrl>>,
    pub files: sqlx::types::Json<Vec<DbModelFile>>,
    pub dependencies: sqlx::types::Json<Vec<String>>,
    pub installation_notes: Option<String>,
    pub usage_examples: sqlx::types::Json<Vec<String>>,
    pub license_text: Option<String>,
    pub model_card: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 数据库下载链接 - 对应 burncloud_service_models::DownloadUrl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbDownloadUrl {
    pub filename: String,
    pub url: String,
    pub size: i64,
    pub checksum: Option<String>,
    pub checksum_algorithm: Option<String>,
    pub is_primary: bool,
}

/// 数据库模型文件 - 对应 burncloud_service_models::ModelFile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbModelFile {
    pub filename: String,
    pub size: i64,
    pub file_type: String, // JSON string of ModelFileType
    pub checksum: Option<String>,
    pub required: bool,
    pub description: Option<String>,
}

/// 数据库全局配置表 - 对应 burncloud_service_models::GlobalConfig
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbGlobalConfig {
    pub id: Uuid,
    pub version: String,
    pub config_data: sqlx::types::Json<serde_json::Value>, // 整个配置作为JSON存储
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 数据库系统指标表 - 对应 burncloud_service_models::SystemMetrics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbSystemMetrics {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f32,
    pub cpu_cores: i32,
    pub memory_total_bytes: i64,
    pub memory_used_bytes: i64,
    pub memory_usage_percent: f32,
    pub disk_total_bytes: i64,
    pub disk_used_bytes: i64,
    pub disk_usage_percent: f32,
    pub network_rx_bytes_per_sec: i64,
    pub network_tx_bytes_per_sec: i64,
    pub gpu_usage_percent: Option<f32>,
    pub gpu_memory_usage_mb: Option<i64>,
    pub load_1m: f32,
    pub load_5m: f32,
    pub load_15m: f32,
}

/// 数据库应用指标表 - 对应 burncloud_service_models::ApplicationMetrics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbApplicationMetrics {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: i64,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub active_connections: i32,
    pub avg_response_time_ms: f32,
    pub p95_response_time_ms: f32,
    pub p99_response_time_ms: f32,
    pub current_qps: f32,
    pub peak_qps: f32,
    pub error_rate_percent: f32,
    pub health_status: String, // JSON string of HealthStatus
}

/// 数据库模型指标表 - 对应 burncloud_service_models::ModelMetrics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbModelMetrics {
    pub id: Uuid,
    pub model_id: Uuid,
    pub runtime_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub status: String, // JSON string of ModelStatus
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub avg_inference_time_ms: f32,
    pub tokens_per_second: f32,
    pub memory_usage_bytes: i64,
    pub gpu_memory_usage_bytes: Option<i64>,
    pub cpu_usage_percent: f32,
    pub gpu_usage_percent: Option<f32>,
    pub queue_length: i32,
    pub last_request_time: Option<DateTime<Utc>>,
}

/// 数据库告警事件表 - 对应 burncloud_service_models::AlertEvent
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbAlertEvent {
    pub id: Uuid,
    pub alert_type: String, // JSON string of AlertType
    pub severity: String, // JSON string of AlertSeverity
    pub title: String,
    pub description: String,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub status: String, // JSON string of AlertStatus
    pub resource_type: String,
    pub resource_id: String,
    pub resource_name: String,
    pub value: f32,
    pub threshold: f32,
    pub labels: sqlx::types::Json<HashMap<String, String>>,
    pub metadata: sqlx::types::Json<HashMap<String, String>>,
}

/// 数据库同步结果表 - 对应 burncloud_service_models::SyncResult
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbSyncResult {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String, // JSON string of SyncStatus
    pub models_added: i32,
    pub models_updated: i32,
    pub models_removed: i32,
    pub error_message: Option<String>,
    pub log_entries: sqlx::types::Json<Vec<String>>,
}

/// 数据库用户会话表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbUserSession {
    pub id: Uuid,
    pub user_id: String,
    pub session_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

/// 数据库API使用统计表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbApiUsage {
    pub id: Uuid,
    pub api_key_id: Option<Uuid>,
    pub endpoint: String,
    pub method: String,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: i32,
    pub status_code: i32,
    pub request_size_bytes: i64,
    pub response_size_bytes: i64,
    pub ip_address: String,
    pub user_agent: Option<String>,
}

/// 数据库任务队列表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbTask {
    pub id: Uuid,
    pub task_type: String,
    pub payload: sqlx::types::Json<serde_json::Value>,
    pub status: String, // pending, running, completed, failed
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub scheduled_at: Option<DateTime<Utc>>,
}

/// 数据库下载任务表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbDownloadTask {
    pub id: Uuid,
    pub model_id: Uuid,
    pub url: String,
    pub file_path: String,
    pub total_size: i64,
    pub downloaded_size: i64,
    pub status: String, // pending, downloading, completed, failed, paused
    pub progress_percent: f32,
    pub download_speed_bps: i64,
    pub estimated_time_remaining: Option<i32>, // seconds
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}