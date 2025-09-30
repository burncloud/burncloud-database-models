//! Unit tests for ModelsService
//!
//! Tests the service layer of database-models including table management,
//! statistics calculation, and business logic integration.

use burncloud_database_models::{
    ModelsService, ModelsTable, BasicModel, BasicModelType, BasicSizeCategory
};
use burncloud_database_core::{create_in_memory_database};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

/// Test fixture for creating basic models
fn create_test_basic_model(name: &str, model_type: BasicModelType, file_size: u64) -> BasicModel {
    BasicModel {
        id: Uuid::new_v4(),
        name: name.to_string(),
        display_name: format!("{} Display", name),
        description: Some(format!("Description for {}", name)),
        version: "1.0.0".to_string(),
        model_type,
        size_category: burncloud_database_models::file_size_to_category(file_size),
        file_size,
        provider: "TestProvider".to_string(),
        license: Some("MIT".to_string()),
        tags: vec!["test".to_string(), "sample".to_string()],
        languages: vec!["English".to_string()],
        file_path: None,
        checksum: None,
        download_url: Some("https://example.com/model".to_string()),
        config: std::collections::HashMap::new(),
        rating: None,
        download_count: 0,
        is_official: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Setup test service
async fn setup_test_service() -> ModelsService {
    let db = Arc::new(create_in_memory_database().await.unwrap());
    ModelsService::new(db).await.unwrap()
}

#[tokio::test]
async fn test_service_creation_and_initialization() {
    let service = setup_test_service().await;

    // Service should initialize successfully
    // Tables should be created automatically
    let stats = service.get_statistics().await.unwrap();
    assert_eq!(stats.total_models, 0);
    assert_eq!(stats.installed_count, 0);
    assert_eq!(stats.official_count, 0);
}

#[tokio::test]
async fn test_service_table_auto_creation() {
    let db = Arc::new(create_in_memory_database().await.unwrap());

    // Tables should not exist initially
    // Service creation should create them automatically
    let service = ModelsService::new(db.clone()).await.unwrap();

    // Should be able to perform operations immediately
    let models = service.repository().get_all_models().await.unwrap();
    assert_eq!(models.len(), 0);
}

#[tokio::test]
async fn test_basic_model_operations() {
    let service = setup_test_service().await;

    // Create test model
    let basic_model = create_test_basic_model("test-model", BasicModelType::Chat, 5_000_000_000);
    let model_table: ModelsTable = basic_model.clone().try_into().unwrap();

    // Create model
    let created = service.repository().create_model(&model_table).await.unwrap();
    assert_eq!(created.name, "test-model");
    assert_eq!(created.model_type, "Chat");

    // Retrieve model
    let retrieved = service.repository().get_model_by_id(basic_model.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test-model");
}

#[tokio::test]
async fn test_statistics_calculation() {
    let service = setup_test_service().await;

    // Initially empty
    let initial_stats = service.get_statistics().await.unwrap();
    assert_eq!(initial_stats.total_models, 0);
    assert_eq!(initial_stats.installed_count, 0);
    assert_eq!(initial_stats.official_count, 0);
    assert_eq!(initial_stats.total_size_bytes, 0);
    assert!(initial_stats.models_by_type.is_empty());

    // Create test models
    let models = vec![
        (create_test_basic_model("chat1", BasicModelType::Chat, 1_000_000_000), false),
        (create_test_basic_model("chat2", BasicModelType::Chat, 2_000_000_000), true),
        (create_test_basic_model("code1", BasicModelType::Code, 3_000_000_000), true),
        (create_test_basic_model("text1", BasicModelType::Text, 4_000_000_000), false),
    ];

    for (mut model, is_official) in models {
        model.is_official = is_official;
        let model_table: ModelsTable = model.try_into().unwrap();
        service.repository().create_model(&model_table).await.unwrap();
    }

    // Check updated statistics
    let stats = service.get_statistics().await.unwrap();
    assert_eq!(stats.total_models, 4);
    assert_eq!(stats.official_count, 2);
    assert_eq!(stats.total_size_bytes, 10_000_000_000);

    // Check models by type
    assert_eq!(stats.models_by_type.get("Chat"), Some(&2));
    assert_eq!(stats.models_by_type.get("Code"), Some(&1));
    assert_eq!(stats.models_by_type.get("Text"), Some(&1));
}

#[tokio::test]
async fn test_statistics_with_installed_models() {
    let service = setup_test_service().await;

    // Create and install models
    let model1 = create_test_basic_model("model1", BasicModelType::Chat, 1_000_000_000);
    let model2 = create_test_basic_model("model2", BasicModelType::Code, 2_000_000_000);

    let model1_table: ModelsTable = model1.clone().try_into().unwrap();
    let model2_table: ModelsTable = model2.clone().try_into().unwrap();

    service.repository().create_model(&model1_table).await.unwrap();
    service.repository().create_model(&model2_table).await.unwrap();

    // Install one model
    service.repository().install_model(model1.id, "/opt/model1".to_string()).await.unwrap();

    // Check statistics
    let stats = service.get_statistics().await.unwrap();
    assert_eq!(stats.total_models, 2);
    assert_eq!(stats.installed_count, 1);
    assert_eq!(stats.total_size_bytes, 3_000_000_000);
}

#[tokio::test]
async fn test_file_size_categorization() {
    // Test the file_size_to_category function
    assert_eq!(burncloud_database_models::file_size_to_category(1_000_000_000), BasicSizeCategory::Small); // 1GB
    assert_eq!(burncloud_database_models::file_size_to_category(5_000_000_000), BasicSizeCategory::Medium); // 5GB
    assert_eq!(burncloud_database_models::file_size_to_category(15_000_000_000), BasicSizeCategory::Large); // 15GB
    assert_eq!(burncloud_database_models::file_size_to_category(50_000_000_000), BasicSizeCategory::XLarge); // 50GB
}

#[tokio::test]
async fn test_basic_model_to_table_conversion() {
    let basic_model = create_test_basic_model("test-conversion", BasicModelType::Embedding, 2_500_000_000);

    // Convert to ModelsTable
    let model_table: Result<ModelsTable, _> = basic_model.clone().try_into();
    assert!(model_table.is_ok());

    let table = model_table.unwrap();
    assert_eq!(table.id, basic_model.id);
    assert_eq!(table.name, basic_model.name);
    assert_eq!(table.display_name, basic_model.display_name);
    assert_eq!(table.model_type, "Embedding");
    assert_eq!(table.file_size, 2_500_000_000);
    assert_eq!(table.provider, basic_model.provider);
    assert_eq!(table.is_official, basic_model.is_official);
}

#[tokio::test]
async fn test_table_to_basic_model_conversion() {
    let service = setup_test_service().await;

    // Create a model table directly
    let mut model_table = ModelsTable::new(
        "conversion-test".to_string(),
        "Conversion Test Model".to_string(),
        "1.0.0".to_string(),
        "Multimodal".to_string(),
        "ConversionProvider".to_string(),
        7_500_000_000,
    );
    model_table.description = Some("Test description".to_string());
    model_table.rating = Some(4.2);

    // Create in database
    let created = service.repository().create_model(&model_table).await.unwrap();

    // Convert to BasicModel
    let basic_model: Result<BasicModel, _> = created.try_into();
    assert!(basic_model.is_ok());

    let basic = basic_model.unwrap();
    assert_eq!(basic.name, "conversion-test");
    assert_eq!(basic.display_name, "Conversion Test Model");
    assert_eq!(basic.model_type, BasicModelType::Multimodal);
    assert_eq!(basic.file_size, 7_500_000_000);
    assert_eq!(basic.description, Some("Test description".to_string()));
    assert_eq!(basic.rating, Some(4.2));
}

#[tokio::test]
async fn test_json_field_handling() {
    let service = setup_test_service().await;

    let mut basic_model = create_test_basic_model("json-test", BasicModelType::Chat, 1_000_000_000);

    // Set complex JSON data
    basic_model.tags = vec!["ai".to_string(), "chat".to_string(), "large".to_string()];
    basic_model.languages = vec!["English".to_string(), "Spanish".to_string(), "French".to_string()];
    basic_model.config = {
        let mut config = std::collections::HashMap::new();
        config.insert("temperature".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()));
        config.insert("max_tokens".to_string(), serde_json::Value::Number(serde_json::Number::from(2048)));
        config.insert("model_type".to_string(), serde_json::Value::String("causal".to_string()));
        config
    };

    let model_table: ModelsTable = basic_model.clone().try_into().unwrap();
    let created = service.repository().create_model(&model_table).await.unwrap();

    // Retrieve and verify JSON fields
    let retrieved = service.repository().get_model_by_id(basic_model.id).await.unwrap().unwrap();

    // Parse JSON fields
    let tags: Vec<String> = serde_json::from_str(&retrieved.tags).unwrap();
    let languages: Vec<String> = serde_json::from_str(&retrieved.languages).unwrap();
    let config: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(&retrieved.config).unwrap();

    assert_eq!(tags, vec!["ai", "chat", "large"]);
    assert_eq!(languages, vec!["English", "Spanish", "French"]);
    assert_eq!(config.len(), 3);
    assert!(config.contains_key("temperature"));
    assert!(config.contains_key("max_tokens"));
    assert!(config.contains_key("model_type"));
}

#[tokio::test]
async fn test_edge_cases_and_optional_fields() {
    let service = setup_test_service().await;

    // Test model with minimal required fields
    let mut minimal_model = create_test_basic_model("minimal", BasicModelType::Text, 500_000_000);
    minimal_model.description = None;
    minimal_model.license = None;
    minimal_model.file_path = None;
    minimal_model.checksum = None;
    minimal_model.download_url = None;
    minimal_model.rating = None;
    minimal_model.tags = vec![];
    minimal_model.languages = vec![];
    minimal_model.config = std::collections::HashMap::new();

    let model_table: ModelsTable = minimal_model.clone().try_into().unwrap();
    let created = service.repository().create_model(&model_table).await.unwrap();

    assert_eq!(created.name, "minimal");
    assert!(created.description.is_none() || created.description == Some("".to_string()));
    assert!(created.license.is_none() || created.license == Some("".to_string()));
    assert!(created.rating.is_none());

    // Test model with maximum field values
    let mut maximal_model = create_test_basic_model("maximal", BasicModelType::Image, u64::MAX);
    maximal_model.description = Some("A".repeat(1000)); // Long description
    maximal_model.rating = Some(5.0);
    maximal_model.download_count = u64::MAX;

    let model_table: ModelsTable = maximal_model.clone().try_into().unwrap();
    let created = service.repository().create_model(&model_table).await.unwrap();

    assert_eq!(created.name, "maximal");
    assert_eq!(created.rating, Some(5.0));
}

#[tokio::test]
async fn test_concurrent_service_operations() {
    let service = Arc::new(setup_test_service().await);

    // Perform concurrent operations
    let mut handles = vec![];

    for i in 0..20 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let model = create_test_basic_model(
                &format!("concurrent-{}", i),
                if i % 2 == 0 { BasicModelType::Chat } else { BasicModelType::Code },
                1_000_000_000 + i as u64,
            );
            let model_table: ModelsTable = model.try_into().unwrap();
            service_clone.repository().create_model(&model_table).await
        });
        handles.push(handle);
    }

    // Wait for all operations
    let mut success_count = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 20);

    // Verify final state
    let stats = service.get_statistics().await.unwrap();
    assert_eq!(stats.total_models, 20);
}

#[tokio::test]
async fn test_service_error_handling() {
    let service = setup_test_service().await;

    // Test duplicate name handling
    let model1 = create_test_basic_model("duplicate-name", BasicModelType::Chat, 1_000_000_000);
    let model2 = create_test_basic_model("duplicate-name", BasicModelType::Code, 2_000_000_000);

    let table1: ModelsTable = model1.try_into().unwrap();
    let table2: ModelsTable = model2.try_into().unwrap();

    // First should succeed
    let result1 = service.repository().create_model(&table1).await;
    assert!(result1.is_ok());

    // Second should fail
    let result2 = service.repository().create_model(&table2).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_statistics_consistency() {
    let service = setup_test_service().await;

    // Create various models
    let models = vec![
        ("chat1", BasicModelType::Chat, 1_000_000_000, true),
        ("chat2", BasicModelType::Chat, 2_000_000_000, false),
        ("code1", BasicModelType::Code, 3_000_000_000, true),
        ("text1", BasicModelType::Text, 4_000_000_000, false),
        ("embed1", BasicModelType::Embedding, 500_000_000, true),
    ];

    for (name, model_type, size, is_official) in models {
        let mut model = create_test_basic_model(name, model_type, size);
        model.is_official = is_official;
        let table: ModelsTable = model.try_into().unwrap();
        service.repository().create_model(&table).await.unwrap();
    }

    let stats = service.get_statistics().await.unwrap();

    // Verify consistency
    assert_eq!(stats.total_models, 5);
    assert_eq!(stats.official_count, 3);
    assert_eq!(stats.total_size_bytes, 10_500_000_000);

    // Verify type counts
    let total_by_type: usize = stats.models_by_type.values().sum();
    assert_eq!(total_by_type, stats.total_models);

    assert_eq!(stats.models_by_type.get("Chat"), Some(&2));
    assert_eq!(stats.models_by_type.get("Code"), Some(&1));
    assert_eq!(stats.models_by_type.get("Text"), Some(&1));
    assert_eq!(stats.models_by_type.get("Embedding"), Some(&1));
}

#[tokio::test]
async fn test_database_persistence_across_operations() {
    let db = Arc::new(create_in_memory_database().await.unwrap());

    // Create service and add data
    {
        let service = ModelsService::new(db.clone()).await.unwrap();
        let model = create_test_basic_model("persistent-model", BasicModelType::Audio, 1_500_000_000);
        let table: ModelsTable = model.try_into().unwrap();
        service.repository().create_model(&table).await.unwrap();
    }

    // Create new service instance with same database
    {
        let service2 = ModelsService::new(db.clone()).await.unwrap();
        let stats = service2.get_statistics().await.unwrap();
        assert_eq!(stats.total_models, 1);

        let models = service2.repository().get_all_models().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "persistent-model");
        assert_eq!(models[0].model_type, "Audio");
    }
}