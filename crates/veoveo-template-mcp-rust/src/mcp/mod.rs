//! The server's MCP surface. Each tool and read checks its permission against the
//! requirement table, and every item belongs to the caller's tenant from the forwarded
//! identity.
mod declarations;
mod requirements;
mod resources;
mod setup;

use rmcp::{
    ErrorData as McpError, RoleServer,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ReadResourceRequestParams, Resource},
    service::RequestContext,
    tool, tool_router,
};
use veoveo_hub_sdk::{
    HubInternalIdentity,
    hosting::{DomainAddress, DomainRead, DomainServer, product_result},
    server_contract::McpServerSetup,
};

use crate::{
    contract::{CreateItemRequest, CreatedItem, ITEM_FORMAT, ItemView, TemplateResource},
    store::{Item, ItemStore, ItemsError, stored_tenant},
};

pub use declarations::{REGISTRATION, registration};
pub use requirements::AUTHORIZATION;
pub use setup::TemplateContract;

use requirements::{Target, require};
use setup::SERVER_SETUP;

/// Build and check the MCP setup, so a bad declaration stops startup.
pub fn force_setup() {
    std::sync::LazyLock::force(&SERVER_SETUP);
}

#[derive(Clone)]
pub struct TemplateMcp {
    items: ItemStore,
    tool_router: ToolRouter<TemplateMcp>,
}

#[tool_router]
impl TemplateMcp {
    pub fn new(items: ItemStore) -> Self {
        Self {
            items,
            tool_router: Self::tools(),
        }
    }

    /// Every tool the server serves.
    pub(crate) fn tools() -> ToolRouter<Self> {
        Self::tool_router()
    }

    #[tool(
        title = "Create an item",
        description = "Create an item in your tenant. The result's `resultUri` is the item's address, `template://item/{item_id}`; list your items at `template://items`.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<CreatedItem>(),
        annotations(read_only_hint = false, destructive_hint = false, idempotent_hint = false, open_world_hint = false)
    )]
    async fn create_item(
        &self,
        Parameters(request): Parameters<CreateItemRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let identity = require(&context, Target::Tool("create_item"))?;
        let item = self
            .items
            .create(tenant(&identity)?, &request.name)
            .await
            .map_err(items_error)?;
        let view = view(item);
        let link = Resource::new(view.item_uri.to_string(), view.name.as_str());
        product_result(
            format!("Created item {}", view.name),
            link,
            &CreatedItem {
                format: ITEM_FORMAT.to_owned(),
                result_uri: view.item_uri.clone(),
                item: view,
            },
        )
    }
}

impl DomainServer for TemplateMcp {
    type Contract = TemplateContract;

    fn setup() -> &'static McpServerSetup<TemplateContract> {
        &SERVER_SETUP
    }

    fn tool_router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    async fn read(
        &self,
        address: DomainAddress<TemplateContract>,
        request: &ReadResourceRequestParams,
        context: &RequestContext<RoleServer>,
    ) -> Result<DomainRead, McpError> {
        resources::read(&self.items, address, request, context).await
    }
}

/// The stored tenant of the caller's items.
fn tenant(identity: &HubInternalIdentity) -> Result<veoveo_store::TenantId, McpError> {
    stored_tenant(&identity.authority.tenant).map_err(|_| {
        McpError::invalid_request("the forwarded identity names no stored tenant", None)
    })
}

/// An item as reads and results return it.
fn view(item: Item) -> ItemView {
    ItemView {
        item_uri: TemplateResource::Item(item.item_id),
        item_id: item.item_id,
        name: item.name,
    }
}

/// A refusal names its rule; a failure of the database is logged and reported without
/// detail.
fn items_error(error: ItemsError) -> McpError {
    match error {
        ItemsError::Cursor(_) => McpError::invalid_params(error.to_string(), None),
        failure @ (ItemsError::Corrupt(_) | ItemsError::Database(_)) => {
            tracing::error!(%failure, "a template request failed");
            McpError::internal_error("the server could not complete the request", None)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_setup_and_documents_admit() {
        std::sync::LazyLock::force(&super::SERVER_SETUP);
        assert!(super::setup::SERVER_DOCS.agent_manual().is_some());
    }
}
