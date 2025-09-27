use async_trait::async_trait;
use sqlx::{Database, Pool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use burncloud_service_models as service;
use crate::models::*;
use crate::converters::*;

/// 数据库仓库特质 - 定义通用的数据库操作接口
#[async_trait]
pub trait DatabaseRepository<DB: Database> {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn pool(&self) -> &Pool<DB>;
}

/// 模型数据库仓库
#[async_trait]
pub trait ModelRepository<DB: Database>: DatabaseRepository<DB> {
    /// 获取所有模型
    async fn get_all_models(&self) -> Result<Vec<service::Model>, Self::Error>;

    /// 根据ID获取模型
    async fn get_model_by_id(&self, id: Uuid) -> Result<Option<service::Model>, Self::Error>;

    /// 根据名称获取模型
    async fn get_model_by_name(&self, name: &str) -> Result<Option<service::Model>, Self::Error>;

    /// 创建新模型
    async fn create_model(&self, model: &service::Model) -> Result<(), Self::Error>;

    /// 更新模型
    async fn update_model(&self, model: &service::Model) -> Result<(), Self::Error>;

    /// 删除模型
    async fn delete_model(&self, id: Uuid) -> Result<(), Self::Error>;

    /// 搜索模型
    async fn search_models(&self, query: &str, limit: Option<i64>) -> Result<Vec<service::Model>, Self::Error>;

    /// 按类型获取模型
    async fn get_models_by_type(&self, model_type: &service::ModelType) -> Result<Vec<service::Model>, Self::Error>;

    /// 按提供商获取模型
    async fn get_models_by_provider(&self, provider: &str) -> Result<Vec<service::Model>, Self::Error>;
}

/// 已安装模型数据库仓库
#[async_trait]
pub trait InstalledModelRepository<DB: Database>: DatabaseRepository<DB> {
    /// 获取所有已安装模型
    async fn get_all_installed_models(&self) -> Result<Vec<service::InstalledModel>, Self::Error>;

    /// 根据模型ID获取已安装模型
    async fn get_installed_model_by_model_id(&self, model_id: Uuid) -> Result<Option<service::InstalledModel>, Self::Error>;

    /// 安装模型
    async fn install_model(&self, installed_model: &service::InstalledModel) -> Result<(), Self::Error>;

    /// 更新已安装模型
    async fn update_installed_model(&self, installed_model: &service::InstalledModel) -> Result<(), Self::Error>;

    /// 卸载模型
    async fn uninstall_model(&self, model_id: Uuid) -> Result<(), Self::Error>;

    /// 按状态获取已安装模型
    async fn get_installed_models_by_status(&self, status: &service::ModelStatus) -> Result<Vec<service::InstalledModel>, Self::Error>;

    /// 更新模型使用统计
    async fn update_model_usage(&self, model_id: Uuid) -> Result<(), Self::Error>;
}

/// 运行时数据库仓库
#[async_trait]
pub trait RuntimeRepository<DB: Database>: DatabaseRepository<DB> {
    /// 获取所有运行时配置
    async fn get_all_runtime_configs(&self) -> Result<Vec<service::RuntimeConfig>, Self::Error>;

    /// 创建运行时配置
    async fn create_runtime_config(&self, config: &service::RuntimeConfig) -> Result<Uuid, Self::Error>;

    /// 获取模型运行时
    async fn get_model_runtime(&self, model_id: Uuid) -> Result<Option<service::ModelRuntime>, Self::Error>;

    /// 创建模型运行时
    async fn create_model_runtime(&self, runtime: &service::ModelRuntime) -> Result<(), Self::Error>;

    /// 更新模型运行时
    async fn update_model_runtime(&self, runtime: &service::ModelRuntime) -> Result<(), Self::Error>;

    /// 删除模型运行时
    async fn delete_model_runtime(&self, runtime_id: Uuid) -> Result<(), Self::Error>;

    /// 记录运行时指标
    async fn record_runtime_metrics(&self, metrics: &service::RuntimeMetrics) -> Result<(), Self::Error>;

    /// 获取运行时指标历史
    async fn get_runtime_metrics_history(
        &self,
        runtime_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<service::RuntimeMetrics>, Self::Error>;

    /// 记录运行时事件
    async fn record_runtime_event(&self, event: &service::RuntimeEvent) -> Result<(), Self::Error>;

    /// 获取运行时事件
    async fn get_runtime_events(
        &self,
        runtime_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<service::RuntimeEvent>, Self::Error>;
}

/// 仓库管理数据库仓库
#[async_trait]
pub trait RepositoryManagementRepository<DB: Database>: DatabaseRepository<DB> {
    /// 获取所有模型仓库
    async fn get_all_repositories(&self) -> Result<Vec<service::ModelRepository>, Self::Error>;

    /// 根据ID获取仓库
    async fn get_repository_by_id(&self, id: Uuid) -> Result<Option<service::ModelRepository>, Self::Error>;

    /// 创建仓库
    async fn create_repository(&self, repository: &service::ModelRepository) -> Result<(), Self::Error>;

    /// 更新仓库
    async fn update_repository(&self, repository: &service::ModelRepository) -> Result<(), Self::Error>;

    /// 删除仓库
    async fn delete_repository(&self, id: Uuid) -> Result<(), Self::Error>;

    /// 记录同步结果
    async fn record_sync_result(&self, sync_result: &service::SyncResult) -> Result<(), Self::Error>;

    /// 获取同步历史
    async fn get_sync_history(&self, repository_id: Uuid, limit: Option<i64>) -> Result<Vec<service::SyncResult>, Self::Error>;
}

/// 监控数据库仓库
#[async_trait]
pub trait MonitoringRepository<DB: Database>: DatabaseRepository<DB> {
    /// 记录系统指标
    async fn record_system_metrics(&self, metrics: &service::SystemMetrics) -> Result<(), Self::Error>;

    /// 获取系统指标历史
    async fn get_system_metrics_history(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<service::SystemMetrics>, Self::Error>;

    /// 记录应用指标
    async fn record_application_metrics(&self, metrics: &service::ApplicationMetrics) -> Result<(), Self::Error>;

    /// 获取应用指标历史
    async fn get_application_metrics_history(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<service::ApplicationMetrics>, Self::Error>;

    /// 记录模型指标
    async fn record_model_metrics(&self, metrics: &service::ModelMetrics) -> Result<(), Self::Error>;

    /// 获取模型指标历史
    async fn get_model_metrics_history(
        &self,
        model_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<service::ModelMetrics>, Self::Error>;

    /// 创建告警事件
    async fn create_alert_event(&self, alert: &service::AlertEvent) -> Result<(), Self::Error>;

    /// 更新告警事件
    async fn update_alert_event(&self, alert: &service::AlertEvent) -> Result<(), Self::Error>;

    /// 获取活跃告警
    async fn get_active_alerts(&self) -> Result<Vec<service::AlertEvent>, Self::Error>;

    /// 获取告警历史
    async fn get_alert_history(&self, limit: Option<i64>) -> Result<Vec<service::AlertEvent>, Self::Error>;
}

/// 配置数据库仓库
#[async_trait]
pub trait ConfigRepository<DB: Database>: DatabaseRepository<DB> {
    /// 获取全局配置
    async fn get_global_config(&self) -> Result<Option<service::GlobalConfig>, Self::Error>;

    /// 保存全局配置
    async fn save_global_config(&self, config: &service::GlobalConfig) -> Result<(), Self::Error>;

    /// 获取配置历史
    async fn get_config_history(&self, limit: Option<i64>) -> Result<Vec<service::GlobalConfig>, Self::Error>;
}

/// 任务管理数据库仓库
#[async_trait]
pub trait TaskRepository<DB: Database>: DatabaseRepository<DB> {
    /// 创建任务
    async fn create_task(&self, task: &DbTask) -> Result<(), Self::Error>;

    /// 获取待处理任务
    async fn get_pending_tasks(&self, limit: Option<i64>) -> Result<Vec<DbTask>, Self::Error>;

    /// 更新任务状态
    async fn update_task_status(&self, task_id: Uuid, status: &str) -> Result<(), Self::Error>;

    /// 完成任务
    async fn complete_task(&self, task_id: Uuid, result: Option<&str>) -> Result<(), Self::Error>;

    /// 失败任务
    async fn fail_task(&self, task_id: Uuid, error: &str) -> Result<(), Self::Error>;

    /// 创建下载任务
    async fn create_download_task(&self, task: &DbDownloadTask) -> Result<(), Self::Error>;

    /// 更新下载进度
    async fn update_download_progress(
        &self,
        task_id: Uuid,
        downloaded_size: i64,
        progress_percent: f32,
        speed_bps: i64,
    ) -> Result<(), Self::Error>;

    /// 获取下载任务
    async fn get_download_tasks_by_model(&self, model_id: Uuid) -> Result<Vec<DbDownloadTask>, Self::Error>;
}

/// 用户会话数据库仓库
#[async_trait]
pub trait SessionRepository<DB: Database>: DatabaseRepository<DB> {
    /// 创建用户会话
    async fn create_session(&self, session: &DbUserSession) -> Result<(), Self::Error>;

    /// 根据令牌获取会话
    async fn get_session_by_token(&self, token: &str) -> Result<Option<DbUserSession>, Self::Error>;

    /// 更新会话最后访问时间
    async fn update_session_last_accessed(&self, session_id: Uuid) -> Result<(), Self::Error>;

    /// 删除会话
    async fn delete_session(&self, session_id: Uuid) -> Result<(), Self::Error>;

    /// 清理过期会话
    async fn cleanup_expired_sessions(&self) -> Result<i64, Self::Error>;

    /// 记录API使用
    async fn record_api_usage(&self, usage: &DbApiUsage) -> Result<(), Self::Error>;

    /// 获取API使用统计
    async fn get_api_usage_stats(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<DbApiUsage>, Self::Error>;
}

/// 通用错误类型
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
}

/// 分页参数
#[derive(Debug, Clone)]
pub struct Pagination {
    pub offset: i64,
    pub limit: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 20,
        }
    }
}

/// 查询过滤器
#[derive(Debug, Clone)]
pub struct QueryFilter {
    pub search: Option<String>,
    pub model_type: Option<service::ModelType>,
    pub provider: Option<String>,
    pub status: Option<service::ModelStatus>,
    pub tags: Vec<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            search: None,
            model_type: None,
            provider: None,
            status: None,
            tags: Vec::new(),
            created_after: None,
            created_before: None,
        }
    }
}

/// 排序选项
#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct SortBy {
    pub field: String,
    pub order: SortOrder,
}

impl Default for SortBy {
    fn default() -> Self {
        Self {
            field: "created_at".to_string(),
            order: SortOrder::Desc,
        }
    }
}

/// 查询选项
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    pub pagination: Pagination,
    pub filter: QueryFilter,
    pub sort_by: SortBy,
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub has_more: bool,
}

impl<T> QueryResult<T> {
    pub fn new(items: Vec<T>, total_count: i64, pagination: &Pagination) -> Self {
        let has_more = (pagination.offset + pagination.limit) < total_count;
        Self {
            items,
            total_count,
            has_more,
        }
    }
}