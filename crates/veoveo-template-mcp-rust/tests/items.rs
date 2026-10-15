//! The item store against a fresh PostgreSQL database migrated as the server's own
//! login. Requires the PostgreSQL 18 server programs on `PATH`.
use std::time::Duration;

use veoveo_fixtures::store::TestDb;
use veoveo_store::deterministic_tenant_id;
use veoveo_template_mcp_rust::{
    contract::{ItemId, ItemName, ListCursor},
    migrator, server,
    store::{ItemPageSize, ItemStore, ItemsError},
};

#[tokio::test]
async fn items_page_in_key_order_and_cursors_stay_with_their_tenant() {
    tokio::time::timeout(Duration::from_secs(60), async {
        let db = TestDb::new().await;
        db.migrate_server(&server(), migrator()).await;
        let items = ItemStore::new(db.server_store(&server()).await);
        let tenant = deterministic_tenant_id("acme").unwrap();
        let other = deterministic_tenant_id("globex").unwrap();
        let mut created = Vec::new();
        for name in ["first", "second", "third"] {
            let name = ItemName::parse(name).unwrap();
            created.push(items.create(tenant, &name).await.unwrap());
        }
        let elsewhere = items
            .create(other, &ItemName::parse("elsewhere").unwrap())
            .await
            .unwrap();

        // Three items page as two and one, and the last page has no cursor.
        let two = ItemPageSize::new(2).unwrap();
        let first = items.page(tenant, None, two).await.unwrap();
        assert_eq!(first.items, created[..2]);
        let next = ListCursor::parse(first.next.expect("a third item follows").as_str()).unwrap();
        let second = items.page(tenant, Some(&next), two).await.unwrap();
        assert_eq!(second.items, created[2..]);
        assert!(second.next.is_none());

        // Another tenant's listing refuses the cursor instead of paging its own rows.
        assert!(matches!(
            items.page(other, Some(&next), two).await,
            Err(ItemsError::Cursor(_))
        ));

        // An exact read sees only the tenant's own item.
        assert_eq!(
            items.item(tenant, created[0].item_id).await.unwrap(),
            Some(created[0].clone())
        );
        assert_eq!(items.item(tenant, elsewhere.item_id).await.unwrap(), None);
        assert_eq!(items.item(tenant, ItemId::new()).await.unwrap(), None);
    })
    .await
    .expect("the item store test exceeded 60 seconds");
}
