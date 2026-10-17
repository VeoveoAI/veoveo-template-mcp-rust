//! The MCP surface through the SDK's in-process Hub against a migrated database:
//! discovery, permission refusals, `create_item` and the reads of its product. The test
//! user is `user-1` of `tenant-a`. Requires the PostgreSQL 18 server programs on `PATH`.
use std::time::Duration;

use serde_json::{Value, json};
use veoveo_fixtures::store::TestDb;
use veoveo_hub_sdk::hosting::{
    Hosted,
    testing::{self, TestHub},
};
use veoveo_template_mcp_rust::{
    contract::TemplatePermission::{self, ItemsRead, ItemsWrite},
    mcp::TemplateMcp,
    migrator, server,
    store::ItemStore,
};

/// A Hub that grants `granted` in front of a server on `db`.
async fn hub(db: &TestDb, granted: &[TemplatePermission]) -> TestHub {
    let items = ItemStore::new(db.server_store(&server()).await);
    TestHub::new(
        testing::for_domain::<TemplateMcp>()
            .handler(move || Hosted::new(TemplateMcp::new(items.clone())))
            .build(),
    )
    .granting(granted.iter().map(|permission| permission.name()))
}

fn error_message(response: &Value) -> &str {
    response["error"]["message"].as_str().unwrap_or_default()
}

fn read_json(response: &Value) -> Value {
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("no contents: {response}"));
    serde_json::from_str(text).unwrap()
}

#[tokio::test]
async fn items_are_created_read_and_listed_with_their_permissions() {
    tokio::time::timeout(Duration::from_secs(60), async {
        let db = TestDb::new().await;
        db.migrate_server(&server(), migrator()).await;
        let full = hub(&db, &[ItemsRead, ItemsWrite]).await;

        let tools = full.rpc("tools/list", json!({})).await;
        let names: Vec<&str> = tools["result"]["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect();
        assert_eq!(names, ["create_item"]);

        let created = full
            .rpc(
                "tools/call",
                json!({"name": "create_item", "arguments": {"name": "Quarterly report"}}),
            )
            .await;
        let product = &created["result"]["structuredContent"];
        let result_uri = product["resultUri"].as_str().expect("resultUri").to_owned();
        assert!(result_uri.starts_with("template://item/"), "{created}");
        assert_eq!(product["item"]["name"], "Quarterly report");

        let item = read_json(&full.rpc("resources/read", json!({"uri": result_uri})).await);
        assert_eq!(item["name"], "Quarterly report");
        assert_eq!(item["itemUri"], result_uri.as_str());

        let listing = read_json(
            &full
                .rpc("resources/read", json!({"uri": "template://items"}))
                .await,
        );
        assert_eq!(listing["items"].as_array().unwrap().len(), 1);
        assert_eq!(listing["nextCursor"], Value::Null);

        // A name the schema refuses reaches no handler.
        let refused = full
            .rpc(
                "tools/call",
                json!({"name": "create_item", "arguments": {"name": " padded "}}),
            )
            .await;
        assert!(
            refused["result"]["isError"] == true || refused["error"].is_object(),
            "{refused}"
        );

        // Reading without the write permission cannot create, and the refusal names the
        // missing permission.
        let reader = hub(&db, &[ItemsRead]).await;
        let denied = reader
            .rpc(
                "tools/call",
                json!({"name": "create_item", "arguments": {"name": "denied"}}),
            )
            .await;
        assert!(
            error_message(&denied).contains("template.items.write"),
            "{denied}"
        );
        let listing = read_json(
            &reader
                .rpc("resources/read", json!({"uri": "template://items"}))
                .await,
        );
        assert_eq!(listing["items"].as_array().unwrap().len(), 1);

        // An unknown item reads as missing.
        let missing = reader
            .rpc(
                "resources/read",
                json!({"uri": "template://item/0190f3a2-7b1c-7d2e-8f3a-4b5c6d7e8f90"}),
            )
            .await;
        assert!(
            error_message(&missing).contains("no such item"),
            "{missing}"
        );
    })
    .await
    .expect("the MCP test exceeded 60 seconds");
}
