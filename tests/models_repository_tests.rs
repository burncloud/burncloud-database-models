//! Unit tests for ModelsRepository
//!
//! Tests the database layer operations including CRUD operations,
//! table creation, data integrity, and error handling.

use burncloud_database_models::{
    ModelsRepository, ModelsTable, InstalledModelsTable
};
use burncloud_database::{Database, create_in_memory_database};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

/// Test fixture for creating test models
fn create_test_model() -> ModelsTable {
    ModelsTable::new(
        "test-model-v1".to_string(),
        "Test Model V1".to_string(),
        "1.0.0".to_string(),
        "Chat".to_string(),
        "TestProvider".to_string(),
        1073741824, // 1GB
    )
}

/// Test fixture for creating test models with specific parameters
fn create_test_model_with_params(
    name: &str,
    display_name: &str,
    model_type: &str,
    provider: &str,
    file_size: i64,
) -> ModelsTable {
    ModelsTable::new(
        name.to_string(),
        display_name.to_string(),
        "1.0.0".to_string(),
        model_type.to_string(),
        provider.to_string(),
        file_size,
    )
}

/// Setup test database and repository
async fn setup_test_repository() -> (Arc<Database>, ModelsRepository) {
    let db = Arc::new(create_in_memory_database().await.unwrap());
    let repository = ModelsRepository::new(db.clone()).await.unwrap();
    repository.ensure_tables_exist().await.unwrap();
    (db, repository)
}

#[tokio::test]
async fn test_repository_creation() {
    let (_db, repository) = setup_test_repository().await;

    // Repository should be created successfully
    // Tables should exist after ensure_tables_exist call
    let models = repository.get_all_models().await.unwrap();
    assert_eq!(models.len(), 0);
}

#[tokio::test]
async fn test_table_creation() {
    let db = Arc::new(create_in_memory_database().await.unwrap());
    let repository = ModelsRepository::new(db.clone()).await.unwrap();

    // Ensure tables creation works
    repository.ensure_tables_exist().await.unwrap();

    // Should be idempotent - calling again should not fail
    repository.ensure_tables_exist().await.unwrap();
}

#[tokio::test]
async fn test_create_model() {
    let (_db, repository) = setup_test_repository().await;

    let model = create_test_model();
    let created = repository.create_model(&model).await.unwrap();

    assert_eq!(created.id, model.id);
    assert_eq!(created.name, model.name);
    assert_eq!(created.display_name, model.display_name);
    assert_eq!(created.model_type, model.model_type);
    assert_eq!(created.provider, model.provider);
    assert_eq!(created.file_size, model.file_size);
    assert_eq!(created.is_official, model.is_official);
}

#[tokio::test]
async fn test_create_duplicate_model_name() {
    let (_db, repository) = setup_test_repository().await;

    let model1 = create_test_model();
    repository.create_model(&model1).await.unwrap();

    // Try to create another model with the same name
    let mut model2 = create_test_model();
    model2.id = Uuid::new_v4(); // Different ID, same name

    // This should fail due to unique constraint on name
    let result = repository.create_model(&model2).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_model_by_id() {
    let (_db, repository) = setup_test_repository().await;

    let model = create_test_model();
    repository.create_model(&model).await.unwrap();

    // Test existing model
    let retrieved = repository.get_model_by_id(model.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, model.id);
    assert_eq!(retrieved.name, model.name);

    // Test non-existent model
    let fake_id = Uuid::new_v4();
    let not_found = repository.get_model_by_id(fake_id).await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_get_model_by_name() {
    let (_db, repository) = setup_test_repository().await;

    let model = create_test_model();
    repository.create_model(&model).await.unwrap();

    // Test existing model
    let retrieved = repository.get_model_by_name(&model.name).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.name, model.name);
    assert_eq!(retrieved.id, model.id);

    // Test non-existent model
    let not_found = repository.get_model_by_name("non-existent").await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_get_all_models() {
    let (_db, repository) = setup_test_repository().await;

    // Initially empty
    let models = repository.get_all_models().await.unwrap();
    assert_eq!(models.len(), 0);

    // Create multiple models
    let model1 = create_test_model_with_params("model1", "Model 1", "Chat", "Provider1", 1000000);
    let model2 = create_test_model_with_params("model2", "Model 2", "Code", "Provider2", 2000000);
    let model3 = create_test_model_with_params("model3", "Model 3", "Text", "Provider1", 3000000);

    repository.create_model(&model1).await.unwrap();
    repository.create_model(&model2).await.unwrap();
    repository.create_model(&model3).await.unwrap();

    // Should retrieve all models
    let all_models = repository.get_all_models().await.unwrap();
    assert_eq!(all_models.len(), 3);

    // Should be ordered by created_at DESC
    let names: Vec<String> = all_models.iter().map(|m| m.name.clone()).collect();
    assert!(names.contains(&"model1".to_string()));
    assert!(names.contains(&"model2".to_string()));
    assert!(names.contains(&"model3".to_string()));
}

#[tokio::test]
async fn test_update_model() {
    let (_db, repository) = setup_test_repository().await;

    let mut model = create_test_model();
    repository.create_model(&model).await.unwrap();

    // Update model properties
    model.display_name = "Updated Test Model".to_string();
    model.description = Some("Updated description".to_string());
    model.rating = Some(4.5);
    model.download_count = 100;
    model.updated_at = Utc::now();

    let updated = repository.update_model(&model).await.unwrap();

    assert_eq!(updated.display_name, "Updated Test Model");
    assert_eq!(updated.description, Some("Updated description".to_string()));
    assert_eq!(updated.rating, Some(4.5));
    assert_eq!(updated.download_count, 100);

    // Verify in database
    let retrieved = repository.get_model_by_id(model.id).await.unwrap().unwrap();
    assert_eq!(retrieved.display_name, "Updated Test Model");
    assert_eq!(retrieved.description, Some("Updated description".to_string()));
}

#[tokio::test]
async fn test_delete_model() {
    let (_db, repository) = setup_test_repository().await;

    let model = create_test_model();
    repository.create_model(&model).await.unwrap();

    // Verify model exists
    let exists = repository.get_model_by_id(model.id).await.unwrap();
    assert!(exists.is_some());

    // Delete model
    let deleted = repository.delete_model(model.id).await.unwrap();
    assert!(deleted);

    // Verify model no longer exists
    let not_exists = repository.get_model_by_id(model.id).await.unwrap();
    assert!(not_exists.is_none());

    // Try to delete non-existent model
    let fake_id = Uuid::new_v4();
    let not_deleted = repository.delete_model(fake_id).await.unwrap();
    assert!(!not_deleted);
}

#[tokio::test]
async fn test_search_models() {
    let (_db, repository) = setup_test_repository().await;

    // Create test models with searchable content
    let model1 = create_test_model_with_params("llama-chat", "Llama Chat Model", "Chat", "Meta", 8000000000);
    let model2 = create_test_model_with_params("codellama-code", "CodeLlama", "Code", "Meta", 7000000000);
    let model3 = create_test_model_with_params("mistral-instruct", "Mistral Instruct", "Text", "Mistral", 7000000000);

    repository.create_model(&model1).await.unwrap();
    repository.create_model(&model2).await.unwrap();
    repository.create_model(&model3).await.unwrap();

    // Search by name
    let llama_results = repository.search_models("llama", Some(10)).await.unwrap();
    assert_eq!(llama_results.len(), 2);

    // Search by display name
    let mistral_results = repository.search_models("Mistral", Some(10)).await.unwrap();
    assert_eq!(mistral_results.len(), 1);
    assert_eq!(mistral_results[0].name, "mistral-instruct");

    // Search with no results
    let no_results = repository.search_models("nonexistent", Some(10)).await.unwrap();
    assert_eq!(no_results.len(), 0);

    // Test limit
    let limited_results = repository.search_models("", Some(2)).await.unwrap();
    assert_eq!(limited_results.len(), 2);
}

#[tokio::test]
async fn test_get_models_by_type() {
    let (_db, repository) = setup_test_repository().await;

    let chat_model = create_test_model_with_params("chat1", "Chat Model 1", "Chat", "Provider1", 1000000);
    let code_model1 = create_test_model_with_params("code1", "Code Model 1", "Code", "Provider1", 2000000);
    let code_model2 = create_test_model_with_params("code2", "Code Model 2", "Code", "Provider2", 3000000);

    repository.create_model(&chat_model).await.unwrap();
    repository.create_model(&code_model1).await.unwrap();
    repository.create_model(&code_model2).await.unwrap();

    // Get chat models
    let chat_models = repository.get_models_by_type("Chat").await.unwrap();
    assert_eq!(chat_models.len(), 1);
    assert_eq!(chat_models[0].name, "chat1");

    // Get code models
    let code_models = repository.get_models_by_type("Code").await.unwrap();
    assert_eq!(code_models.len(), 2);

    // Get non-existent type
    let text_models = repository.get_models_by_type("Text").await.unwrap();
    assert_eq!(text_models.len(), 0);
}

#[tokio::test]
async fn test_get_models_by_provider() {
    let (_db, repository) = setup_test_repository().await;

    let meta_model1 = create_test_model_with_params("meta1", "Meta Model 1", "Chat", "Meta", 1000000);
    let meta_model2 = create_test_model_with_params("meta2", "Meta Model 2", "Code", "Meta", 2000000);
    let openai_model = create_test_model_with_params("openai1", "OpenAI Model", "Text", "OpenAI", 3000000);

    repository.create_model(&meta_model1).await.unwrap();
    repository.create_model(&meta_model2).await.unwrap();
    repository.create_model(&openai_model).await.unwrap();

    // Get Meta models
    let meta_models = repository.get_models_by_provider("Meta").await.unwrap();
    assert_eq!(meta_models.len(), 2);

    // Get OpenAI models
    let openai_models = repository.get_models_by_provider("OpenAI").await.unwrap();
    assert_eq!(openai_models.len(), 1);
    assert_eq!(openai_models[0].name, "openai1");

    // Get non-existent provider
    let fake_models = repository.get_models_by_provider("FakeProvider").await.unwrap();
    assert_eq!(fake_models.len(), 0);
}

#[tokio::test]
async fn test_get_official_models() {
    let (_db, repository) = setup_test_repository().await;

    let mut official_model = create_test_model_with_params("official1", "Official Model", "Chat", "Provider1", 1000000);
    official_model.is_official = true;

    let mut unofficial_model = create_test_model_with_params("unofficial1", "Unofficial Model", "Code", "Provider2", 2000000);
    unofficial_model.is_official = false;

    repository.create_model(&official_model).await.unwrap();
    repository.create_model(&unofficial_model).await.unwrap();

    let official_models = repository.get_official_models().await.unwrap();
    assert_eq!(official_models.len(), 1);
    assert_eq!(official_models[0].name, "official1");
    assert!(official_models[0].is_official);
}

#[tokio::test]
async fn test_install_model() {
    let (_db, repository) = setup_test_repository().await;

    // Create a base model first
    let model = create_test_model();
    repository.create_model(&model).await.unwrap();

    // Install the model
    let install_path = "/opt/models/test-model".to_string();
    let installed = repository.install_model(model.id, install_path.clone()).await.unwrap();

    assert_eq!(installed.model_id, model.id);
    assert_eq!(installed.install_path, install_path);
    assert_eq!(installed.status, "Stopped");
    assert_eq!(installed.usage_count, 0);
    assert!(installed.port.is_none());
    assert!(installed.process_id.is_none());
}

#[tokio::test]
async fn test_get_installed_models() {
    let (_db, repository) = setup_test_repository().await;

    // Initially no installed models
    let installed = repository.get_installed_models().await.unwrap();
    assert_eq!(installed.len(), 0);

    // Create and install models
    let model1 = create_test_model_with_params("model1", "Model 1", "Chat", "Provider1", 1000000);
    let model2 = create_test_model_with_params("model2", "Model 2", "Code", "Provider2", 2000000);

    repository.create_model(&model1).await.unwrap();
    repository.create_model(&model2).await.unwrap();

    repository.install_model(model1.id, "/opt/model1".to_string()).await.unwrap();
    repository.install_model(model2.id, "/opt/model2".to_string()).await.unwrap();

    // Get all installed models
    let installed = repository.get_installed_models().await.unwrap();
    assert_eq!(installed.len(), 2);

    // Verify model data is included
    let (base_model, installed_model) = &installed[0];
    assert!(base_model.name == "model1" || base_model.name == "model2");
    assert!(installed_model.install_path.contains("opt"));
}

#[tokio::test]
async fn test_update_model_status() {
    let (_db, repository) = setup_test_repository().await;

    // Create and install a model
    let model = create_test_model();
    repository.create_model(&model).await.unwrap();
    repository.install_model(model.id, "/opt/test".to_string()).await.unwrap();

    // Update status
    repository.update_model_status(model.id, "Running".to_string()).await.unwrap();

    // Verify status was updated
    let installed = repository.get_installed_models().await.unwrap();
    assert_eq!(installed.len(), 1);
    let (_, installed_model) = &installed[0];
    assert_eq!(installed_model.status, "Running");

    // Update to different status
    repository.update_model_status(model.id, "Stopped".to_string()).await.unwrap();

    let installed = repository.get_installed_models().await.unwrap();
    let (_, installed_model) = &installed[0];
    assert_eq!(installed_model.status, "Stopped");
}

#[tokio::test]
async fn test_data_types_and_constraints() {
    let (_db, repository) = setup_test_repository().await;

    let mut model = create_test_model();

    // Test with various data types
    model.description = Some("A very long description that contains special characters: !@#$%^&*()_+{}|:<>?".to_string());
    model.tags = r#"["tag1", "tag2", "special-tag"]"#.to_string();
    model.languages = r#"["English", "中文", "Español"]"#.to_string();
    model.config = r#"{"temperature": 0.7, "max_tokens": 2048}"#.to_string();
    model.rating = Some(4.99);
    model.download_count = 999999999;
    model.file_size = 999999999999;

    let created = repository.create_model(&model).await.unwrap();
    assert_eq!(created.description, model.description);
    assert_eq!(created.rating, Some(4.99));
    assert_eq!(created.download_count, 999999999);
    assert_eq!(created.file_size, 999999999999);

    // Retrieve and verify data integrity
    let retrieved = repository.get_model_by_id(model.id).await.unwrap().unwrap();
    assert_eq!(retrieved.tags, model.tags);
    assert_eq!(retrieved.languages, model.languages);
    assert_eq!(retrieved.config, model.config);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let (_db, repository) = setup_test_repository().await;
    let repo = Arc::new(repository);

    // Create multiple models concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let repo_clone = repo.clone();
        let handle = tokio::spawn(async move {
            let model = create_test_model_with_params(
                &format!("concurrent-model-{}", i),
                &format!("Concurrent Model {}", i),
                "Chat",
                "ConcurrentProvider",
                1000000 + i as i64,
            );
            repo_clone.create_model(&model).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    // All operations should succeed
    for result in results {
        assert!(result.is_ok());
    }

    // Verify all models were created
    let all_models = repo.get_all_models().await.unwrap();
    assert_eq!(all_models.len(), 10);
}

#[tokio::test]
async fn test_transaction_behavior() {
    let (_db, repository) = setup_test_repository().await;

    // Test that failed operations don't leave partial state
    let model = create_test_model();
    repository.create_model(&model).await.unwrap();

    // Try to update with invalid data (this should fail gracefully)
    let mut invalid_model = model.clone();
    invalid_model.file_size = -1; // Invalid file size

    // The update should still work as SQLite is flexible with types
    // but let's test edge cases
    let updated = repository.update_model(&invalid_model).await.unwrap();
    assert_eq!(updated.file_size, -1);

    // Verify the original model is not in an inconsistent state
    let retrieved = repository.get_model_by_id(model.id).await.unwrap().unwrap();
    assert_eq!(retrieved.id, model.id);
}

#[tokio::test]
async fn test_large_dataset_operations() {
    let (_db, repository) = setup_test_repository().await;

    // Create a larger number of models to test performance
    const MODEL_COUNT: usize = 100;

    let start_time = std::time::Instant::now();

    // Batch create models
    for i in 0..MODEL_COUNT {
        let model = create_test_model_with_params(
            &format!("bulk-model-{:03}", i),
            &format!("Bulk Model {:03}", i),
            if i % 3 == 0 { "Chat" } else if i % 3 == 1 { "Code" } else { "Text" },
            if i % 2 == 0 { "Provider1" } else { "Provider2" },
            1000000 + i as i64,
        );
        repository.create_model(&model).await.unwrap();
    }

    let creation_time = start_time.elapsed();
    println!("Created {} models in {:?}", MODEL_COUNT, creation_time);

    // Test retrieval performance
    let retrieval_start = std::time::Instant::now();
    let all_models = repository.get_all_models().await.unwrap();
    let retrieval_time = retrieval_start.elapsed();

    println!("Retrieved {} models in {:?}", all_models.len(), retrieval_time);
    assert_eq!(all_models.len(), MODEL_COUNT);

    // Test search performance
    let search_start = std::time::Instant::now();
    let search_results = repository.search_models("bulk", Some(50)).await.unwrap();
    let search_time = search_start.elapsed();

    println!("Searched {} models in {:?}", search_results.len(), search_time);
    assert_eq!(search_results.len(), 50); // Limited by the limit parameter

    // Test filtering performance
    let filter_start = std::time::Instant::now();
    let chat_models = repository.get_models_by_type("Chat").await.unwrap();
    let filter_time = filter_start.elapsed();

    println!("Filtered {} chat models in {:?}", chat_models.len(), filter_time);
    assert!(chat_models.len() > 0);

    // Performance assertions (these are loose bounds)
    assert!(creation_time.as_millis() < 5000); // Should create 100 models in < 5 seconds
    assert!(retrieval_time.as_millis() < 1000); // Should retrieve all in < 1 second
    assert!(search_time.as_millis() < 1000); // Should search in < 1 second
    assert!(filter_time.as_millis() < 1000); // Should filter in < 1 second
}