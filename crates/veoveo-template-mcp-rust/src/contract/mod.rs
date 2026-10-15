//! The server's public types. Consumers select `default-features = false,
//! features = ["contract"]`, which builds without a database or runtime dependency.

mod ids;
mod items;
mod permissions;
mod resource;

pub use ids::{
    ItemId, ItemName, ListCursor, MAX_CURSOR_BYTES, MAX_ITEM_NAME_CHARS, TemplateIdError,
};
pub use items::{CreateItemRequest, CreatedItem, ITEM_FORMAT, ItemListing, ItemView};
pub use permissions::{TemplatePermission, TemplateScope};
pub use resource::{TemplateDocument, TemplateResource, TemplateResourceError};
