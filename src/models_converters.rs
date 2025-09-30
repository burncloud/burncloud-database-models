use crate::models_table::{ModelsTable, InstalledModelsTable};
use serde_json;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Basic types that are shared across layers without dependencies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicModelType {
    Chat,
    Code,
    Text,
    Embedding,
    Image,
    Audio,
    Video,
    Multimodal,
    Other,
}

impl std::str::FromStr for BasicModelType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Chat" => Ok(BasicModelType::Chat),
            "Code" => Ok(BasicModelType::Code),
            "Text" => Ok(BasicModelType::Text),
            "Embedding" => Ok(BasicModelType::Embedding),
            "Image" => Ok(BasicModelType::Image),
            "Audio" => Ok(BasicModelType::Audio),
            "Video" => Ok(BasicModelType::Video),
            "Multimodal" => Ok(BasicModelType::Multimodal),
            "Other" => Ok(BasicModelType::Other),
            _ => Err(format!("Invalid model type: {}", s)),
        }
    }
}

impl std::fmt::Display for BasicModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BasicModelType::Chat => write!(f, "Chat"),
            BasicModelType::Code => write!(f, "Code"),
            BasicModelType::Text => write!(f, "Text"),
            BasicModelType::Embedding => write!(f, "Embedding"),
            BasicModelType::Image => write!(f, "Image"),
            BasicModelType::Audio => write!(f, "Audio"),
            BasicModelType::Video => write!(f, "Video"),
            BasicModelType::Multimodal => write!(f, "Multimodal"),
            BasicModelType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicSizeCategory {
    Small,
    Medium,
    Large,
    XLarge,
}

impl std::fmt::Display for BasicSizeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BasicSizeCategory::Small => write!(f, "Small"),
            BasicSizeCategory::Medium => write!(f, "Medium"),
            BasicSizeCategory::Large => write!(f, "Large"),
            BasicSizeCategory::XLarge => write!(f, "XLarge"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicModelStatus {
    Running,
    Starting,
    Stopping,
    Stopped,
    Error,
}

impl std::str::FromStr for BasicModelStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Running" => Ok(BasicModelStatus::Running),
            "Starting" => Ok(BasicModelStatus::Starting),
            "Stopping" => Ok(BasicModelStatus::Stopping),
            "Stopped" => Ok(BasicModelStatus::Stopped),
            "Error" => Ok(BasicModelStatus::Error),
            _ => Err(format!("Invalid model status: {}", s)),
        }
    }
}

impl std::fmt::Display for BasicModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BasicModelStatus::Running => write!(f, "Running"),
            BasicModelStatus::Starting => write!(f, "Starting"),
            BasicModelStatus::Stopping => write!(f, "Stopping"),
            BasicModelStatus::Stopped => write!(f, "Stopped"),
            BasicModelStatus::Error => write!(f, "Error"),
        }
    }
}

/// Basic model structure without service layer dependencies
#[derive(Debug, Clone)]
pub struct BasicModel {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub version: String,
    pub model_type: BasicModelType,
    pub size_category: BasicSizeCategory,
    pub file_size: u64,
    pub provider: String,
    pub license: Option<String>,
    pub tags: Vec<String>,
    pub languages: Vec<String>,
    pub file_path: Option<String>,
    pub checksum: Option<String>,
    pub download_url: Option<String>,
    pub config: HashMap<String, serde_json::Value>,
    pub rating: Option<f32>,
    pub download_count: u64,
    pub is_official: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Basic installed model structure
#[derive(Debug, Clone)]
pub struct BasicInstalledModel {
    pub id: Uuid,
    pub model: BasicModel,
    pub install_path: String,
    pub installed_at: DateTime<Utc>,
    pub status: BasicModelStatus,
    pub port: Option<u32>,
    pub process_id: Option<u32>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Convert BasicModel to database ModelsTable
impl TryFrom<BasicModel> for ModelsTable {
    type Error = String;

    fn try_from(basic_model: BasicModel) -> Result<Self, Self::Error> {
        Ok(ModelsTable {
            id: basic_model.id,
            name: basic_model.name,
            display_name: basic_model.display_name,
            description: basic_model.description,
            version: basic_model.version,
            model_type: basic_model.model_type.to_string(),
            size_category: basic_model.size_category.to_string(),
            file_size: basic_model.file_size as i64,
            provider: basic_model.provider,
            license: basic_model.license,
            tags: serde_json::to_string(&basic_model.tags)
                .map_err(|e| format!("Failed to serialize tags: {}", e))?,
            languages: serde_json::to_string(&basic_model.languages)
                .map_err(|e| format!("Failed to serialize languages: {}", e))?,
            file_path: basic_model.file_path,
            checksum: basic_model.checksum,
            download_url: basic_model.download_url,
            config: serde_json::to_string(&basic_model.config)
                .map_err(|e| format!("Failed to serialize config: {}", e))?,
            rating: basic_model.rating,
            download_count: basic_model.download_count as i64,
            is_official: basic_model.is_official,
            created_at: basic_model.created_at,
            updated_at: basic_model.updated_at,
        })
    }
}

/// Convert database ModelsTable to BasicModel
impl TryFrom<ModelsTable> for BasicModel {
    type Error = String;

    fn try_from(db_model: ModelsTable) -> Result<Self, Self::Error> {
        let tags: Vec<String> = serde_json::from_str(&db_model.tags)
            .map_err(|e| format!("Failed to parse tags: {}", e))?;

        let languages: Vec<String> = serde_json::from_str(&db_model.languages)
            .map_err(|e| format!("Failed to parse languages: {}", e))?;

        let config: HashMap<String, serde_json::Value> = serde_json::from_str(&db_model.config)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        let model_type = db_model.model_type.parse::<BasicModelType>()
            .map_err(|e| format!("Invalid model type: {}", e))?;

        let size_category = match db_model.size_category.as_str() {
            "Small" => BasicSizeCategory::Small,
            "Medium" => BasicSizeCategory::Medium,
            "Large" => BasicSizeCategory::Large,
            "XLarge" => BasicSizeCategory::XLarge,
            _ => return Err(format!("Invalid size category: {}", db_model.size_category)),
        };

        Ok(BasicModel {
            id: db_model.id,
            name: db_model.name,
            display_name: db_model.display_name,
            description: db_model.description,
            version: db_model.version,
            model_type,
            size_category,
            file_size: db_model.file_size as u64,
            provider: db_model.provider,
            license: db_model.license,
            tags,
            languages,
            file_path: db_model.file_path,
            checksum: db_model.checksum,
            download_url: db_model.download_url,
            config,
            rating: db_model.rating,
            download_count: db_model.download_count as u64,
            is_official: db_model.is_official,
            created_at: db_model.created_at,
            updated_at: db_model.updated_at,
        })
    }
}

/// Convert BasicInstalledModel to database InstalledModelsTable
impl TryFrom<BasicInstalledModel> for InstalledModelsTable {
    type Error = String;

    fn try_from(basic_installed: BasicInstalledModel) -> Result<Self, Self::Error> {
        Ok(InstalledModelsTable {
            id: basic_installed.id,
            model_id: basic_installed.model.id,
            install_path: basic_installed.install_path,
            installed_at: basic_installed.installed_at,
            status: basic_installed.status.to_string(),
            port: basic_installed.port.map(|p| p as i32),
            process_id: basic_installed.process_id.map(|p| p as i32),
            last_used: basic_installed.last_used,
            usage_count: basic_installed.usage_count as i64,
            created_at: basic_installed.created_at,
            updated_at: basic_installed.updated_at,
        })
    }
}

/// Convert database records to BasicInstalledModel
pub fn db_to_basic_installed_model((db_model, db_installed): (ModelsTable, InstalledModelsTable)) -> Result<BasicInstalledModel, String> {
    let basic_model = BasicModel::try_from(db_model)?;

    let status = db_installed.status.parse::<BasicModelStatus>()
        .map_err(|e| format!("Invalid model status: {}", e))?;

    Ok(BasicInstalledModel {
        id: db_installed.id,
        model: basic_model,
        install_path: db_installed.install_path,
        installed_at: db_installed.installed_at,
        status,
        port: db_installed.port.map(|p| p as u32),
        process_id: db_installed.process_id.map(|p| p as u32),
        last_used: db_installed.last_used,
        usage_count: db_installed.usage_count as u64,
        created_at: db_installed.created_at,
        updated_at: db_installed.updated_at,
    })
}

/// Convert size in bytes to size category
pub fn file_size_to_category(size_bytes: u64) -> BasicSizeCategory {
    if size_bytes < 3_000_000_000 {       // 3GB
        BasicSizeCategory::Small
    } else if size_bytes < 10_000_000_000 { // 10GB
        BasicSizeCategory::Medium
    } else if size_bytes < 25_000_000_000 { // 25GB
        BasicSizeCategory::Large
    } else {
        BasicSizeCategory::XLarge
    }
}

/// Convert size category to approximate bytes (for display purposes)
pub fn category_to_approximate_size(category: BasicSizeCategory) -> (u64, &'static str) {
    match category {
        BasicSizeCategory::Small => (1_500_000_000, "~1.5GB"),    // 1.5GB
        BasicSizeCategory::Medium => (5_500_000_000, "~5.5GB"),   // 5.5GB
        BasicSizeCategory::Large => (19_000_000_000, "~19GB"),    // 19GB
        BasicSizeCategory::XLarge => (50_000_000_000, "~50GB+"),  // 50GB+
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_model_to_db_conversion() {
        let now = Utc::now();
        let mut config = HashMap::new();
        config.insert("temperature".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()));

        let basic_model = BasicModel {
            id: Uuid::new_v4(),
            name: "test-model".to_string(),
            display_name: "Test Model".to_string(),
            description: Some("A test model".to_string()),
            version: "1.0.0".to_string(),
            model_type: BasicModelType::Chat,
            size_category: BasicSizeCategory::Small,
            file_size: 1024 * 1024 * 1024, // 1GB
            provider: "Test Provider".to_string(),
            license: Some("MIT".to_string()),
            tags: vec!["ai".to_string(), "chat".to_string()],
            languages: vec!["en".to_string(), "zh".to_string()],
            file_path: Some("/path/to/model".to_string()),
            checksum: Some("abc123".to_string()),
            download_url: Some("https://example.com/model".to_string()),
            config,
            rating: Some(4.5),
            download_count: 100,
            is_official: true,
            created_at: now,
            updated_at: now,
        };

        let db_model: ModelsTable = basic_model.clone().try_into().unwrap();

        assert_eq!(db_model.name, "test-model");
        assert_eq!(db_model.display_name, "Test Model");
        assert_eq!(db_model.model_type, "Chat");
        assert_eq!(db_model.size_category, "Small");
        assert_eq!(db_model.file_size, 1024 * 1024 * 1024);
        assert_eq!(db_model.is_official, true);
    }

    #[test]
    fn test_db_model_to_basic_conversion() {
        let now = Utc::now();
        let db_model = ModelsTable {
            id: Uuid::new_v4(),
            name: "test-model".to_string(),
            display_name: "Test Model".to_string(),
            description: Some("A test model".to_string()),
            version: "1.0.0".to_string(),
            model_type: "Chat".to_string(),
            size_category: "Small".to_string(),
            file_size: 1024 * 1024 * 1024,
            provider: "Test Provider".to_string(),
            license: Some("MIT".to_string()),
            tags: r#"["ai", "chat"]"#.to_string(),
            languages: r#"["en", "zh"]"#.to_string(),
            file_path: Some("/path/to/model".to_string()),
            checksum: Some("abc123".to_string()),
            download_url: Some("https://example.com/model".to_string()),
            config: r#"{"temperature": 0.7}"#.to_string(),
            rating: Some(4.5),
            download_count: 100,
            is_official: true,
            created_at: now,
            updated_at: now,
        };

        let basic_model: BasicModel = db_model.try_into().unwrap();

        assert_eq!(basic_model.name, "test-model");
        assert_eq!(basic_model.display_name, "Test Model");
        assert_eq!(basic_model.model_type, BasicModelType::Chat);
        assert_eq!(basic_model.size_category, BasicSizeCategory::Small);
        assert_eq!(basic_model.tags, vec!["ai", "chat"]);
        assert_eq!(basic_model.languages, vec!["en", "zh"]);
        assert_eq!(basic_model.is_official, true);
    }

    #[test]
    fn test_file_size_to_category() {
        assert_eq!(file_size_to_category(1_000_000_000), BasicSizeCategory::Small);     // 1GB
        assert_eq!(file_size_to_category(5_000_000_000), BasicSizeCategory::Medium);    // 5GB
        assert_eq!(file_size_to_category(15_000_000_000), BasicSizeCategory::Large);    // 15GB
        assert_eq!(file_size_to_category(50_000_000_000), BasicSizeCategory::XLarge);   // 50GB
    }

    #[test]
    fn test_category_to_approximate_size() {
        let (size, desc) = category_to_approximate_size(BasicSizeCategory::Small);
        assert_eq!(desc, "~1.5GB");
        assert!(size > 0);

        let (size, desc) = category_to_approximate_size(BasicSizeCategory::XLarge);
        assert_eq!(desc, "~50GB+");
        assert!(size > 40_000_000_000);
    }
}