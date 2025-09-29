use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Models 表的数据库结构
/// 对应数据库中的 models 表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelsTable {
    /// 模型ID - 主键
    pub id: Uuid,
    /// 模型名称 - 唯一
    pub name: String,
    /// 模型显示名称
    pub display_name: String,
    /// 模型描述
    pub description: Option<String>,
    /// 模型版本
    pub version: String,
    /// 模型类型 (Chat, Code, Text, Embedding, etc.)
    pub model_type: String,
    /// 模型大小分类 (Small, Medium, Large, XLarge)
    pub size_category: String,
    /// 模型文件大小 (字节)
    pub file_size: i64,
    /// 模型提供商
    pub provider: String,
    /// 模型许可证
    pub license: Option<String>,
    /// 模型标签 (JSON数组)
    pub tags: String,
    /// 支持的语言 (JSON数组)
    pub languages: String,
    /// 模型文件路径
    pub file_path: Option<String>,
    /// 模型检验和
    pub checksum: Option<String>,
    /// 下载URL
    pub download_url: Option<String>,
    /// 模型配置参数 (JSON)
    pub config: String,
    /// 模型评分
    pub rating: Option<f32>,
    /// 下载次数
    pub download_count: i64,
    /// 是否为官方模型
    pub is_official: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// Installed Models 表的数据库结构
/// 对应数据库中的 installed_models 表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InstalledModelsTable {
    /// 安装记录ID - 主键
    pub id: Uuid,
    /// 关联的模型ID - 外键
    pub model_id: Uuid,
    /// 安装路径
    pub install_path: String,
    /// 安装时间
    pub installed_at: DateTime<Utc>,
    /// 当前状态 (Running, Stopped, Starting, etc.)
    pub status: String,
    /// 运行端口
    pub port: Option<i32>,
    /// 进程ID
    pub process_id: Option<i32>,
    /// 最后使用时间
    pub last_used: Option<DateTime<Utc>>,
    /// 使用次数
    pub usage_count: i64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl ModelsTable {
    /// 创建新的模型记录
    pub fn new(
        name: String,
        display_name: String,
        version: String,
        model_type: String,
        provider: String,
        file_size: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            display_name,
            description: None,
            version,
            model_type,
            size_category: calculate_size_category(file_size),
            file_size,
            provider,
            license: None,
            tags: "[]".to_string(),
            languages: "[]".to_string(),
            file_path: None,
            checksum: None,
            download_url: None,
            config: "{}".to_string(),
            rating: None,
            download_count: 0,
            is_official: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// 标记为已更新
    pub fn mark_updated(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl InstalledModelsTable {
    /// 创建新的安装记录
    pub fn new(model_id: Uuid, install_path: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            model_id,
            install_path,
            installed_at: now,
            status: "Stopped".to_string(),
            port: None,
            process_id: None,
            last_used: None,
            usage_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// 标记为已使用
    pub fn mark_used(&mut self) {
        self.last_used = Some(Utc::now());
        self.usage_count += 1;
        self.updated_at = Utc::now();
    }

    /// 更新状态
    pub fn update_status(&mut self, status: String) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

/// 根据文件大小计算模型大小分类
fn calculate_size_category(file_size: i64) -> String {
    let size_gb = file_size as f64 / 1024.0 / 1024.0 / 1024.0;
    match size_gb {
        s if s < 3.0 => "Small".to_string(),
        s if s < 8.0 => "Medium".to_string(),
        s if s < 30.0 => "Large".to_string(),
        _ => "XLarge".to_string(),
    }
}

/// Models 表的 SQL 建表语句
pub const CREATE_MODELS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS models (
    id UUID PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    display_name VARCHAR NOT NULL,
    description TEXT,
    version VARCHAR NOT NULL,
    model_type VARCHAR NOT NULL,
    size_category VARCHAR NOT NULL,
    file_size BIGINT NOT NULL,
    provider VARCHAR NOT NULL,
    license VARCHAR,
    tags TEXT NOT NULL DEFAULT '[]',
    languages TEXT NOT NULL DEFAULT '[]',
    file_path VARCHAR,
    checksum VARCHAR,
    download_url VARCHAR,
    config TEXT NOT NULL DEFAULT '{}',
    rating REAL,
    download_count BIGINT NOT NULL DEFAULT 0,
    is_official BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_models_name ON models(name);
CREATE INDEX IF NOT EXISTS idx_models_type ON models(model_type);
CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider);
CREATE INDEX IF NOT EXISTS idx_models_official ON models(is_official);
"#;

/// Installed Models 表的 SQL 建表语句
pub const CREATE_INSTALLED_MODELS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS installed_models (
    id UUID PRIMARY KEY,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    install_path VARCHAR NOT NULL,
    installed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR NOT NULL,
    port INTEGER,
    process_id INTEGER,
    last_used TIMESTAMP WITH TIME ZONE,
    usage_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_installed_models_model_id ON installed_models(model_id);
CREATE INDEX IF NOT EXISTS idx_installed_models_status ON installed_models(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_installed_models_unique_model ON installed_models(model_id);
"#;