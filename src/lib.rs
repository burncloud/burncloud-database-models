//! # BurnCloud Database Models
//!
//! 专门处理 models 表的数据库操作和业务逻辑
//!
//! 这个包负责：
//! - models 表的数据库操作
//! - models 相关的业务逻辑
//! - 类型转换（service models <-> database models）

pub mod models_table;
pub mod models_repository;
pub mod models_service;
pub mod models_converters;

pub use models_table::*;
pub use models_repository::*;
pub use models_service::*;
pub use models_converters::*;

// 重新导出相关类型
pub use burncloud_service_models;
pub use burncloud_database_core;
pub use burncloud_database_client;