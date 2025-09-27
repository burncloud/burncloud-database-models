// SQL 迁移脚本 - 创建 BurnCloud 模型管理数据库表结构

/// PostgreSQL 迁移脚本
pub const POSTGRES_MIGRATIONS: &[&str] = &[
    // 001_initial_schema.sql
    r#"
-- 创建模型表
CREATE TABLE IF NOT EXISTS models (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    version VARCHAR(100) NOT NULL,
    model_type VARCHAR(50) NOT NULL,
    size_category VARCHAR(50) NOT NULL,
    file_size BIGINT NOT NULL,
    provider VARCHAR(255) NOT NULL,
    license VARCHAR(255),
    tags JSONB NOT NULL DEFAULT '[]',
    languages JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    file_path VARCHAR(500),
    checksum VARCHAR(255),
    download_url TEXT,
    config JSONB NOT NULL DEFAULT '{}',
    rating REAL,
    download_count BIGINT NOT NULL DEFAULT 0,
    is_official BOOLEAN NOT NULL DEFAULT false
);

-- 创建已安装模型表
CREATE TABLE IF NOT EXISTS installed_models (
    id UUID PRIMARY KEY,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    install_path VARCHAR(500) NOT NULL,
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(50) NOT NULL,
    port INTEGER,
    process_id INTEGER,
    last_used TIMESTAMPTZ,
    usage_count BIGINT NOT NULL DEFAULT 0,
    UNIQUE(model_id)
);

-- 创建可用模型表
CREATE TABLE IF NOT EXISTS available_models (
    id UUID PRIMARY KEY,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    is_installed BOOLEAN NOT NULL DEFAULT false,
    published_at TIMESTAMPTZ NOT NULL,
    last_updated TIMESTAMPTZ NOT NULL,
    system_requirements JSONB NOT NULL,
    UNIQUE(model_id)
);

-- 创建运行时配置表
CREATE TABLE IF NOT EXISTS runtime_configs (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    max_context_length INTEGER,
    temperature REAL,
    top_p REAL,
    top_k INTEGER,
    max_tokens INTEGER,
    stop_sequences JSONB NOT NULL DEFAULT '[]',
    batch_size INTEGER,
    max_concurrent_requests INTEGER,
    gpu_device_ids JSONB NOT NULL DEFAULT '[]',
    memory_limit_mb BIGINT,
    enable_streaming BOOLEAN NOT NULL DEFAULT true,
    custom_params JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建模型运行时表
CREATE TABLE IF NOT EXISTS model_runtimes (
    id UUID PRIMARY KEY,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    runtime_config_id UUID NOT NULL REFERENCES runtime_configs(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL,
    process_id INTEGER,
    started_at TIMESTAMPTZ,
    stopped_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL,
    health_endpoint VARCHAR(255) NOT NULL,
    api_endpoint VARCHAR(255) NOT NULL,
    log_file VARCHAR(500),
    environment JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(model_id, port)
);

-- 创建运行时指标表
CREATE TABLE IF NOT EXISTS runtime_metrics (
    id UUID PRIMARY KEY,
    runtime_id UUID NOT NULL REFERENCES model_runtimes(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cpu_usage_percent REAL NOT NULL,
    memory_usage_mb BIGINT NOT NULL,
    gpu_usage_percent REAL,
    gpu_memory_usage_mb BIGINT,
    active_connections INTEGER NOT NULL,
    total_requests BIGINT NOT NULL,
    successful_requests BIGINT NOT NULL,
    failed_requests BIGINT NOT NULL,
    avg_response_time_ms REAL NOT NULL,
    throughput_rps REAL NOT NULL,
    queue_length INTEGER NOT NULL
);

-- 创建运行时事件表
CREATE TABLE IF NOT EXISTS runtime_events (
    id UUID PRIMARY KEY,
    runtime_id UUID NOT NULL REFERENCES model_runtimes(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    message TEXT NOT NULL,
    details JSONB,
    severity VARCHAR(20) NOT NULL
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_models_name ON models(name);
CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider);
CREATE INDEX IF NOT EXISTS idx_models_model_type ON models(model_type);
CREATE INDEX IF NOT EXISTS idx_models_created_at ON models(created_at);
CREATE INDEX IF NOT EXISTS idx_installed_models_model_id ON installed_models(model_id);
CREATE INDEX IF NOT EXISTS idx_installed_models_status ON installed_models(status);
CREATE INDEX IF NOT EXISTS idx_available_models_model_id ON available_models(model_id);
CREATE INDEX IF NOT EXISTS idx_runtime_metrics_runtime_id ON runtime_metrics(runtime_id);
CREATE INDEX IF NOT EXISTS idx_runtime_metrics_timestamp ON runtime_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_runtime_events_runtime_id ON runtime_events(runtime_id);
CREATE INDEX IF NOT EXISTS idx_runtime_events_timestamp ON runtime_events(timestamp);
"#,

    // 002_repositories.sql
    r#"
-- 创建模型仓库表
CREATE TABLE IF NOT EXISTS model_repositories (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    url TEXT NOT NULL,
    repo_type VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    auth_config JSONB,
    last_sync TIMESTAMPTZ,
    sync_status VARCHAR(50) NOT NULL DEFAULT 'never',
    description TEXT,
    tags JSONB NOT NULL DEFAULT '[]',
    priority INTEGER NOT NULL DEFAULT 100,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建仓库索引表
CREATE TABLE IF NOT EXISTS repository_indexes (
    id UUID PRIMARY KEY,
    repository_id UUID NOT NULL REFERENCES model_repositories(id) ON DELETE CASCADE,
    version VARCHAR(100) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    checksum VARCHAR(255),
    metadata JSONB NOT NULL DEFAULT '{}',
    UNIQUE(repository_id)
);

-- 创建仓库模型表
CREATE TABLE IF NOT EXISTS repository_models (
    id UUID PRIMARY KEY,
    repository_id UUID NOT NULL REFERENCES model_repositories(id) ON DELETE CASCADE,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    repo_model_id VARCHAR(255) NOT NULL,
    repo_path VARCHAR(500) NOT NULL,
    download_urls JSONB NOT NULL DEFAULT '[]',
    files JSONB NOT NULL DEFAULT '[]',
    dependencies JSONB NOT NULL DEFAULT '[]',
    installation_notes TEXT,
    usage_examples JSONB NOT NULL DEFAULT '[]',
    license_text TEXT,
    model_card TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repository_id, repo_model_id)
);

-- 创建同步结果表
CREATE TABLE IF NOT EXISTS sync_results (
    id UUID PRIMARY KEY,
    repository_id UUID NOT NULL REFERENCES model_repositories(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL,
    models_added INTEGER NOT NULL DEFAULT 0,
    models_updated INTEGER NOT NULL DEFAULT 0,
    models_removed INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    log_entries JSONB NOT NULL DEFAULT '[]'
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_repository_models_repository_id ON repository_models(repository_id);
CREATE INDEX IF NOT EXISTS idx_repository_models_model_id ON repository_models(model_id);
CREATE INDEX IF NOT EXISTS idx_sync_results_repository_id ON sync_results(repository_id);
CREATE INDEX IF NOT EXISTS idx_sync_results_started_at ON sync_results(started_at);
"#,

    // 003_monitoring.sql
    r#"
-- 创建全局配置表
CREATE TABLE IF NOT EXISTS global_configs (
    id UUID PRIMARY KEY,
    version VARCHAR(100) NOT NULL,
    config_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建系统指标表
CREATE TABLE IF NOT EXISTS system_metrics (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cpu_usage_percent REAL NOT NULL,
    cpu_cores INTEGER NOT NULL,
    memory_total_bytes BIGINT NOT NULL,
    memory_used_bytes BIGINT NOT NULL,
    memory_usage_percent REAL NOT NULL,
    disk_total_bytes BIGINT NOT NULL,
    disk_used_bytes BIGINT NOT NULL,
    disk_usage_percent REAL NOT NULL,
    network_rx_bytes_per_sec BIGINT NOT NULL,
    network_tx_bytes_per_sec BIGINT NOT NULL,
    gpu_usage_percent REAL,
    gpu_memory_usage_mb BIGINT,
    load_1m REAL NOT NULL,
    load_5m REAL NOT NULL,
    load_15m REAL NOT NULL
);

-- 创建应用指标表
CREATE TABLE IF NOT EXISTS application_metrics (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    uptime_seconds BIGINT NOT NULL,
    total_requests BIGINT NOT NULL,
    successful_requests BIGINT NOT NULL,
    failed_requests BIGINT NOT NULL,
    active_connections INTEGER NOT NULL,
    avg_response_time_ms REAL NOT NULL,
    p95_response_time_ms REAL NOT NULL,
    p99_response_time_ms REAL NOT NULL,
    current_qps REAL NOT NULL,
    peak_qps REAL NOT NULL,
    error_rate_percent REAL NOT NULL,
    health_status VARCHAR(20) NOT NULL
);

-- 创建模型指标表
CREATE TABLE IF NOT EXISTS model_metrics (
    id UUID PRIMARY KEY,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    runtime_id UUID REFERENCES model_runtimes(id) ON DELETE SET NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(50) NOT NULL,
    total_requests BIGINT NOT NULL,
    successful_requests BIGINT NOT NULL,
    failed_requests BIGINT NOT NULL,
    avg_inference_time_ms REAL NOT NULL,
    tokens_per_second REAL NOT NULL,
    memory_usage_bytes BIGINT NOT NULL,
    gpu_memory_usage_bytes BIGINT,
    cpu_usage_percent REAL NOT NULL,
    gpu_usage_percent REAL,
    queue_length INTEGER NOT NULL,
    last_request_time TIMESTAMPTZ
);

-- 创建告警事件表
CREATE TABLE IF NOT EXISTS alert_events (
    id UUID PRIMARY KEY,
    alert_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'triggered',
    resource_type VARCHAR(100) NOT NULL,
    resource_id VARCHAR(255) NOT NULL,
    resource_name VARCHAR(255) NOT NULL,
    value REAL NOT NULL,
    threshold REAL NOT NULL,
    labels JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_system_metrics_timestamp ON system_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_application_metrics_timestamp ON application_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_model_metrics_model_id ON model_metrics(model_id);
CREATE INDEX IF NOT EXISTS idx_model_metrics_timestamp ON model_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_alert_events_triggered_at ON alert_events(triggered_at);
CREATE INDEX IF NOT EXISTS idx_alert_events_status ON alert_events(status);
CREATE INDEX IF NOT EXISTS idx_alert_events_severity ON alert_events(severity);
"#,

    // 004_tasks_and_sessions.sql
    r#"
-- 创建用户会话表
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_accessed TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET NOT NULL,
    user_agent TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- 创建API使用统计表
CREATE TABLE IF NOT EXISTS api_usage (
    id UUID PRIMARY KEY,
    api_key_id UUID,
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    response_time_ms INTEGER NOT NULL,
    status_code INTEGER NOT NULL,
    request_size_bytes BIGINT NOT NULL,
    response_size_bytes BIGINT NOT NULL,
    ip_address INET NOT NULL,
    user_agent TEXT
);

-- 创建任务队列表
CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY,
    task_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    scheduled_at TIMESTAMPTZ
);

-- 创建下载任务表
CREATE TABLE IF NOT EXISTS download_tasks (
    id UUID PRIMARY KEY,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    total_size BIGINT NOT NULL,
    downloaded_size BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    progress_percent REAL NOT NULL DEFAULT 0,
    download_speed_bps BIGINT NOT NULL DEFAULT 0,
    estimated_time_remaining INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_session_token ON user_sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_api_usage_timestamp ON api_usage(timestamp);
CREATE INDEX IF NOT EXISTS idx_api_usage_endpoint ON api_usage(endpoint);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_task_type ON tasks(task_type);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_download_tasks_model_id ON download_tasks(model_id);
CREATE INDEX IF NOT EXISTS idx_download_tasks_status ON download_tasks(status);
"#,

    // 005_triggers_and_functions.sql
    r#"
-- 创建自动更新 updated_at 字段的函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 为需要的表创建触发器
CREATE TRIGGER update_models_updated_at BEFORE UPDATE ON models
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_runtime_configs_updated_at BEFORE UPDATE ON runtime_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_model_runtimes_updated_at BEFORE UPDATE ON model_runtimes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_model_repositories_updated_at BEFORE UPDATE ON model_repositories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_repository_models_updated_at BEFORE UPDATE ON repository_models
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_global_configs_updated_at BEFORE UPDATE ON global_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 创建清理过期数据的函数
CREATE OR REPLACE FUNCTION cleanup_old_metrics()
RETURNS void AS $$
BEGIN
    -- 清理30天前的系统指标
    DELETE FROM system_metrics WHERE timestamp < NOW() - INTERVAL '30 days';

    -- 清理30天前的应用指标
    DELETE FROM application_metrics WHERE timestamp < NOW() - INTERVAL '30 days';

    -- 清理7天前的运行时指标
    DELETE FROM runtime_metrics WHERE timestamp < NOW() - INTERVAL '7 days';

    -- 清理90天前的API使用记录
    DELETE FROM api_usage WHERE timestamp < NOW() - INTERVAL '90 days';

    -- 清理过期的用户会话
    DELETE FROM user_sessions WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

-- 创建计算模型统计信息的函数
CREATE OR REPLACE FUNCTION get_model_stats(model_uuid UUID)
RETURNS JSON AS $$
DECLARE
    result JSON;
BEGIN
    SELECT json_build_object(
        'total_requests', COALESCE(SUM(total_requests), 0),
        'successful_requests', COALESCE(SUM(successful_requests), 0),
        'failed_requests', COALESCE(SUM(failed_requests), 0),
        'avg_inference_time_ms', COALESCE(AVG(avg_inference_time_ms), 0),
        'avg_tokens_per_second', COALESCE(AVG(tokens_per_second), 0),
        'last_activity', MAX(timestamp)
    ) INTO result
    FROM model_metrics
    WHERE model_id = model_uuid
    AND timestamp >= NOW() - INTERVAL '24 hours';

    RETURN result;
END;
$$ LANGUAGE plpgsql;
"#,
];

/// SQLite 迁移脚本
pub const SQLITE_MIGRATIONS: &[&str] = &[
    // 001_initial_schema.sql
    r#"
-- 创建模型表
CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    version TEXT NOT NULL,
    model_type TEXT NOT NULL,
    size_category TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    provider TEXT NOT NULL,
    license TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    languages TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    file_path TEXT,
    checksum TEXT,
    download_url TEXT,
    config TEXT NOT NULL DEFAULT '{}',
    rating REAL,
    download_count INTEGER NOT NULL DEFAULT 0,
    is_official INTEGER NOT NULL DEFAULT 0
);

-- 创建已安装模型表
CREATE TABLE IF NOT EXISTS installed_models (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    install_path TEXT NOT NULL,
    installed_at TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL,
    port INTEGER,
    process_id INTEGER,
    last_used TEXT,
    usage_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE CASCADE,
    UNIQUE(model_id)
);

-- 创建可用模型表
CREATE TABLE IF NOT EXISTS available_models (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    is_installed INTEGER NOT NULL DEFAULT 0,
    published_at TEXT NOT NULL,
    last_updated TEXT NOT NULL,
    system_requirements TEXT NOT NULL,
    FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE CASCADE,
    UNIQUE(model_id)
);

-- 创建运行时配置表
CREATE TABLE IF NOT EXISTS runtime_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    max_context_length INTEGER,
    temperature REAL,
    top_p REAL,
    top_k INTEGER,
    max_tokens INTEGER,
    stop_sequences TEXT NOT NULL DEFAULT '[]',
    batch_size INTEGER,
    max_concurrent_requests INTEGER,
    gpu_device_ids TEXT NOT NULL DEFAULT '[]',
    memory_limit_mb INTEGER,
    enable_streaming INTEGER NOT NULL DEFAULT 1,
    custom_params TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 创建模型运行时表
CREATE TABLE IF NOT EXISTS model_runtimes (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    runtime_config_id TEXT NOT NULL,
    name TEXT NOT NULL,
    port INTEGER NOT NULL,
    process_id INTEGER,
    started_at TEXT,
    stopped_at TEXT,
    status TEXT NOT NULL,
    health_endpoint TEXT NOT NULL,
    api_endpoint TEXT NOT NULL,
    log_file TEXT,
    environment TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE CASCADE,
    FOREIGN KEY (runtime_config_id) REFERENCES runtime_configs(id) ON DELETE CASCADE,
    UNIQUE(model_id, port)
);

-- 创建运行时指标表
CREATE TABLE IF NOT EXISTS runtime_metrics (
    id TEXT PRIMARY KEY,
    runtime_id TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    cpu_usage_percent REAL NOT NULL,
    memory_usage_mb INTEGER NOT NULL,
    gpu_usage_percent REAL,
    gpu_memory_usage_mb INTEGER,
    active_connections INTEGER NOT NULL,
    total_requests INTEGER NOT NULL,
    successful_requests INTEGER NOT NULL,
    failed_requests INTEGER NOT NULL,
    avg_response_time_ms REAL NOT NULL,
    throughput_rps REAL NOT NULL,
    queue_length INTEGER NOT NULL,
    FOREIGN KEY (runtime_id) REFERENCES model_runtimes(id) ON DELETE CASCADE
);

-- 创建运行时事件表
CREATE TABLE IF NOT EXISTS runtime_events (
    id TEXT PRIMARY KEY,
    runtime_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    message TEXT NOT NULL,
    details TEXT,
    severity TEXT NOT NULL,
    FOREIGN KEY (runtime_id) REFERENCES model_runtimes(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_models_name ON models(name);
CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider);
CREATE INDEX IF NOT EXISTS idx_models_model_type ON models(model_type);
CREATE INDEX IF NOT EXISTS idx_models_created_at ON models(created_at);
CREATE INDEX IF NOT EXISTS idx_installed_models_model_id ON installed_models(model_id);
CREATE INDEX IF NOT EXISTS idx_installed_models_status ON installed_models(status);
CREATE INDEX IF NOT EXISTS idx_available_models_model_id ON available_models(model_id);
CREATE INDEX IF NOT EXISTS idx_runtime_metrics_runtime_id ON runtime_metrics(runtime_id);
CREATE INDEX IF NOT EXISTS idx_runtime_metrics_timestamp ON runtime_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_runtime_events_runtime_id ON runtime_events(runtime_id);
CREATE INDEX IF NOT EXISTS idx_runtime_events_timestamp ON runtime_events(timestamp);
"#,

    // 其他 SQLite 迁移脚本可以根据需要添加...
];

/// MySQL 迁移脚本
pub const MYSQL_MIGRATIONS: &[&str] = &[
    // MySQL 迁移脚本可以根据需要添加...
];

use sqlx::{Database, Pool};
use async_trait::async_trait;

#[async_trait]
pub trait MigrationRunner<DB: Database> {
    async fn run_migrations(pool: &Pool<DB>) -> Result<(), sqlx::Error>;
    async fn get_migration_version(pool: &Pool<DB>) -> Result<i32, sqlx::Error>;
}

#[cfg(feature = "postgres")]
pub struct PostgresMigrationRunner;

#[cfg(feature = "postgres")]
#[async_trait]
impl MigrationRunner<sqlx::Postgres> for PostgresMigrationRunner {
    async fn run_migrations(pool: &Pool<sqlx::Postgres>) -> Result<(), sqlx::Error> {
        // 创建迁移历史表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS _migration_history (
                id SERIAL PRIMARY KEY,
                version INTEGER NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#)
        .execute(pool)
        .await?;

        // 获取当前版本
        let current_version = Self::get_migration_version(pool).await.unwrap_or(0);

        // 运行所有高于当前版本的迁移
        for (index, migration) in POSTGRES_MIGRATIONS.iter().enumerate() {
            let version = (index + 1) as i32;
            if version > current_version {
                // 执行迁移
                sqlx::query(migration).execute(pool).await?;

                // 记录迁移历史
                sqlx::query(r#"
                    INSERT INTO _migration_history (version, name)
                    VALUES ($1, $2)
                "#)
                .bind(version)
                .bind(format!("migration_{:03}", version))
                .execute(pool)
                .await?;

                println!("Applied migration version {}", version);
            }
        }

        Ok(())
    }

    async fn get_migration_version(pool: &Pool<sqlx::Postgres>) -> Result<i32, sqlx::Error> {
        let row: Option<(i32,)> = sqlx::query_as(
            "SELECT MAX(version) FROM _migration_history"
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|(v,)| v).unwrap_or(0))
    }
}

#[cfg(feature = "sqlite")]
pub struct SqliteMigrationRunner;

#[cfg(feature = "sqlite")]
#[async_trait]
impl MigrationRunner<sqlx::Sqlite> for SqliteMigrationRunner {
    async fn run_migrations(pool: &Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
        // 创建迁移历史表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS _migration_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL UNIQUE,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#)
        .execute(pool)
        .await?;

        // 获取当前版本
        let current_version = Self::get_migration_version(pool).await.unwrap_or(0);

        // 运行所有高于当前版本的迁移
        for (index, migration) in SQLITE_MIGRATIONS.iter().enumerate() {
            let version = (index + 1) as i32;
            if version > current_version {
                // 执行迁移
                sqlx::query(migration).execute(pool).await?;

                // 记录迁移历史
                sqlx::query(r#"
                    INSERT INTO _migration_history (version, name)
                    VALUES (?1, ?2)
                "#)
                .bind(version)
                .bind(format!("migration_{:03}", version))
                .execute(pool)
                .await?;

                println!("Applied migration version {}", version);
            }
        }

        Ok(())
    }

    async fn get_migration_version(pool: &Pool<sqlx::Sqlite>) -> Result<i32, sqlx::Error> {
        let row: Option<(Option<i32>,)> = sqlx::query_as(
            "SELECT MAX(version) FROM _migration_history"
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.and_then(|(v,)| v).unwrap_or(0))
    }
}