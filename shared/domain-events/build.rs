//! domain-events パッケージのビルドスクリプト
//!
//! このビルドスクリプトは protobuf 定義から Rust のコードを生成します。
//! Effect プロジェクトの各マイクロサービス間で共有されるドメインイベントの
//! メッセージ型を定義し、Event Sourcing パターンの実装に使用されます。

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../protos".to_string();

    let mut config = tonic_prost_build::configure();

    // 生成コードの clippy 警告を抑制
    config = config
        .type_attribute(".", "#[allow(clippy::all)]")
        .type_attribute(".", "#[allow(dead_code)]")
        .type_attribute(".", "#[allow(missing_docs)]");

    // serde サポートを追加
    config = config
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    // Timestamp 型のフィールドには専用の serde ヘルパーを使用
    // 全ての optional Timestamp フィールドに適用
    config = config
        .field_attribute(
            "occurred_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "created_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "updated_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "completed_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "generated_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "next_review_date",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "last_reviewed_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "changed_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "deleted_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "signed_in_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "signed_out_at",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        )
        .field_attribute(
            "new_expiry",
            "#[serde(with = \"crate::serde_helpers::timestamp\")]",
        );

    // サービスビルドを無効化（domain-events はメッセージ型のみ使用）
    config = config.build_server(false).build_client(false);

    config.compile_protos(
        &[
            &format!("{proto_root}/common/events.proto"),
            &format!("{proto_root}/events/learning_events.proto"),
            &format!("{proto_root}/events/vocabulary_events.proto"),
            &format!("{proto_root}/events/algorithm_events.proto"),
            &format!("{proto_root}/events/ai_events.proto"),
            &format!("{proto_root}/events/user_events.proto"),
            &format!("{proto_root}/common/learning_types.proto"),
            &format!("{proto_root}/services/user_service.proto"), // LearningGoal 型のため必要
        ],
        &[&proto_root],
    )?;

    Ok(())
}
