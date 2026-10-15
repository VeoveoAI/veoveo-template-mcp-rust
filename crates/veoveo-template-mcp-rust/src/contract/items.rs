//! Items: the arguments of `create_item`, an item as reads return it, the product
//! `create_item` returns and a page of the item listing.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ItemId, ItemName, ListCursor, TemplateResource};

/// The format of a created item.
pub const ITEM_FORMAT: &str = "https://veoveo.ai/formats/template-item";

/// The arguments of `create_item`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateItemRequest {
    /// The new item's name.
    pub name: ItemName,
}

/// One item of the caller's tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemView {
    pub item_id: ItemId,
    /// `template://item/{item_id}`, where the item stays readable.
    pub item_uri: TemplateResource,
    pub name: ItemName,
}

/// What `create_item` returns: the new item and its address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatedItem {
    /// `https://veoveo.ai/formats/template-item`.
    pub format: String,
    /// `template://item/{item_id}`, the item's address.
    pub result_uri: TemplateResource,
    pub item: ItemView,
}

/// A page of the caller's items in creation order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemListing {
    pub items: Vec<ItemView>,
    /// The cursor of the next page, or `null` on the last page.
    pub next_cursor: Option<ListCursor>,
}
