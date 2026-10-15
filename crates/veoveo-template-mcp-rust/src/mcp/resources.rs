//! Reads of the server's resources. Every read is private to the caller's tenant, and an
//! item another tenant holds reads as missing.
use rmcp::{
    ErrorData as McpError, RoleServer, model::ReadResourceRequestParams, service::RequestContext,
};
use veoveo_sdk_rust::hosting::{DomainRead, json_read, served_by_host};

use crate::{
    contract::{ItemListing, ListCursor, TemplateResource},
    store::{ItemPageSize, ItemStore, MAX_ITEMS_PER_PAGE},
};

use super::{
    items_error,
    requirements::{Target, require},
    tenant, view,
};

pub(super) async fn read(
    items: &ItemStore,
    address: TemplateResource,
    request: &ReadResourceRequestParams,
    context: &RequestContext<RoleServer>,
) -> Result<DomainRead, McpError> {
    let uri = request.uri.as_str();
    let result = match address {
        TemplateResource::Docs | TemplateResource::Document(_) | TemplateResource::Contract => {
            return Err(served_by_host());
        }
        TemplateResource::Items { cursor } => {
            let identity = require(context, Target::Resource(uri))?;
            let size = ItemPageSize::new(MAX_ITEMS_PER_PAGE).expect("the page size is in range");
            let page = items
                .page(tenant(&identity)?, cursor.as_ref(), size)
                .await
                .map_err(items_error)?;
            let next_cursor = page
                .next
                .map(|next| ListCursor::parse(next.as_str()))
                .transpose()
                .map_err(|_| McpError::internal_error("the next cursor is malformed", None))?;
            json_read(
                uri,
                &ItemListing {
                    items: page.items.into_iter().map(view).collect(),
                    next_cursor,
                },
            )?
        }
        TemplateResource::Item(item_id) => {
            let identity = require(context, Target::Resource(uri))?;
            let item = items
                .item(tenant(&identity)?, item_id)
                .await
                .map_err(items_error)?
                .ok_or_else(|| McpError::invalid_params("no such item", None))?;
            json_read(uri, &view(item))?
        }
    };
    Ok(DomainRead::no_store(result))
}
