use tonic::Request;
use uuid::Uuid;

// Proto 定義のインポート
pub mod proto {
    pub mod effect {
        pub mod common {
            tonic::include_proto!("effect.common");
        }
        pub mod services {
            pub mod vocabulary_command {
                tonic::include_proto!("effect.services.vocabulary_command");
            }
        }
    }
    pub use effect::{common::*, services::vocabulary_command::*};
}

use proto::{
    CommandMetadata,
    CreateVocabularyItemRequest,
    DeleteVocabularyItemRequest,
    FieldUpdate,
    UpdateVocabularyItemRequest,
    vocabulary_command_service_client::VocabularyCommandServiceClient,
};

#[tokio::test]
async fn test_vocabulary_item_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // gRPC クライアントを作成
    let mut client = VocabularyCommandServiceClient::connect("http://localhost:50052").await?;

    let user_id = Uuid::new_v4().to_string();
    let metadata = CommandMetadata {
        command_id:     Uuid::new_v4().to_string(),
        correlation_id: Uuid::new_v4().to_string(),
        aggregate_id:   String::new(), // 新規作成時は空
        trace_context:  None,
        issued_by:      user_id.clone(),
        issued_at:      None,
        version:        1,
        source:         "grpc_test".to_string(),
        timeout_ms:     None,
        retry_policy:   None,
    };

    // 1. 語彙項目を作成
    println!("Creating vocabulary item...");
    let create_request = Request::new(CreateVocabularyItemRequest {
        metadata:       Some(metadata.clone()),
        word:           "apple".to_string(),
        definitions:    vec!["A round fruit with red or green skin".to_string()],
        part_of_speech: "noun".to_string(),
        register:       "general".to_string(),
        domain:         "food".to_string(),
    });

    let create_response = client.create_vocabulary_item(create_request).await?;
    let item_id = create_response.into_inner().item_id;
    println!("Created item with ID: {}", item_id);

    // 2. 語彙項目を更新
    println!("Updating vocabulary item...");
    let update_request = Request::new(UpdateVocabularyItemRequest {
        metadata:         Some(metadata.clone()),
        item_id:          item_id.clone(),
        updates:          vec![FieldUpdate {
            field_name: "disambiguation".to_string(),
            value_json: r#""(fruit)""#.to_string(),
        }],
        expected_version: 1,
    });

    let update_response = client.update_vocabulary_item(update_request).await?;
    let new_version = update_response.into_inner().new_version;
    println!("Updated item to version: {}", new_version);

    // 3. 語彙項目を削除
    println!("Deleting vocabulary item...");
    let delete_request = Request::new(DeleteVocabularyItemRequest {
        metadata: Some(metadata),
        item_id:  item_id.clone(),
    });

    client.delete_vocabulary_item(delete_request).await?;
    println!("Deleted item with ID: {}", item_id);

    println!("\n✅ All tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_duplicate_word_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = VocabularyCommandServiceClient::connect("http://localhost:50052").await?;

    let metadata = CommandMetadata {
        command_id:     Uuid::new_v4().to_string(),
        correlation_id: Uuid::new_v4().to_string(),
        aggregate_id:   String::new(),
        trace_context:  None,
        issued_by:      Uuid::new_v4().to_string(),
        issued_at:      None,
        version:        1,
        source:         "grpc_test".to_string(),
        timeout_ms:     None,
        retry_policy:   None,
    };

    // 最初の項目を作成
    let word = format!("test_{}", Uuid::new_v4());
    let create_request = Request::new(CreateVocabularyItemRequest {
        metadata:       Some(metadata.clone()),
        word:           word.clone(),
        definitions:    vec!["First definition".to_string()],
        part_of_speech: "noun".to_string(),
        register:       "general".to_string(),
        domain:         "test".to_string(),
    });

    let first_response = client.create_vocabulary_item(create_request).await?;
    let first_id = first_response.into_inner().item_id;
    println!("Created first item with ID: {}", first_id);

    // 同じ単語で異なる意味の項目を作成（disambiguation で区別）
    let create_request2 = Request::new(CreateVocabularyItemRequest {
        metadata:       Some(metadata),
        word:           word.clone(),
        definitions:    vec!["Second definition".to_string()],
        part_of_speech: "verb".to_string(),
        register:       "general".to_string(),
        domain:         "test".to_string(),
    });

    let second_response = client.create_vocabulary_item(create_request2).await?;
    let second_id = second_response.into_inner().item_id;
    println!("Created second item with ID: {}", second_id);

    assert_ne!(first_id, second_id, "IDs should be different");
    println!("\n✅ Duplicate word handling test passed!");

    Ok(())
}

#[tokio::test]
async fn test_optimistic_locking() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = VocabularyCommandServiceClient::connect("http://localhost:50052").await?;

    let metadata = CommandMetadata {
        command_id:     Uuid::new_v4().to_string(),
        correlation_id: Uuid::new_v4().to_string(),
        aggregate_id:   String::new(),
        trace_context:  None,
        issued_by:      Uuid::new_v4().to_string(),
        issued_at:      None,
        version:        1,
        source:         "grpc_test".to_string(),
        timeout_ms:     None,
        retry_policy:   None,
    };

    // 語彙項目を作成
    let word = format!("lock_test_{}", Uuid::new_v4());
    let create_request = Request::new(CreateVocabularyItemRequest {
        metadata: Some(metadata.clone()),
        word,
        definitions: vec!["Test definition".to_string()],
        part_of_speech: "noun".to_string(),
        register: "general".to_string(),
        domain: "test".to_string(),
    });

    let create_response = client.create_vocabulary_item(create_request).await?;
    let item_id = create_response.into_inner().item_id;

    // 間違ったバージョンで更新を試みる
    let update_request = Request::new(UpdateVocabularyItemRequest {
        metadata:         Some(metadata),
        item_id:          item_id.clone(),
        updates:          vec![FieldUpdate {
            field_name: "disambiguation".to_string(),
            value_json: r#""(test)""#.to_string(),
        }],
        expected_version: 999, // 間違ったバージョン
    });

    match client.update_vocabulary_item(update_request).await {
        Err(status) if status.code() == tonic::Code::Aborted => {
            println!("✅ Optimistic locking correctly prevented concurrent update");
            Ok(())
        },
        Err(e) => Err(format!("Unexpected error: {}", e).into()),
        Ok(_) => Err("Update should have failed with wrong version".into()),
    }
}
