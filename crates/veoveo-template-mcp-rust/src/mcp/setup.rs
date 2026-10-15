//! The server's checked contract: slug, scheme, scope, documents, the fixed resources it
//! lists and the templates it declares.
use std::sync::LazyLock;

use rmcp::model::{Resource, ResourceTemplate, ServerCapabilities, ServerConfig};
use veoveo_sdk_rust::{
    ServerSlug,
    docs::ServerDocs,
    server_contract::{
        McpResource, McpResourceTemplate, McpServerContract, McpServerSetup, McpSetupError,
    },
};
use veoveo_types::{ResourceScheme, ResourceTemplateUri};

use crate::contract::{TemplateDocument, TemplateResource, TemplateScope};

/// The crate documents embedded at build time and served under `template://docs`,
/// `template://docs/{doc_id}`, `template://contract` and the admin document routes.
pub(crate) static SERVER_DOCS: LazyLock<ServerDocs> =
    LazyLock::new(|| veoveo_sdk_rust::server_docs!("template"));

pub struct TemplateContract;

pub(crate) static SERVER_SETUP: LazyLock<McpServerSetup<TemplateContract>> =
    LazyLock::new(|| McpServerSetup::new().expect("the template's MCP contract setup"));

impl McpServerContract for TemplateContract {
    type Scope = TemplateScope;
    type Resource = TemplateResource;

    fn slug() -> ServerSlug {
        crate::server()
    }
    fn scheme() -> ResourceScheme {
        ResourceScheme::parse("template").expect("declared template scheme")
    }
    fn scopes() -> &'static [TemplateScope] {
        <TemplateScope as veoveo_types::Vocabulary>::ALL
    }
    fn documents() -> &'static ServerDocs {
        &SERVER_DOCS
    }
    fn server_config() -> ServerConfig {
        server_config()
    }
    fn resources() -> Result<Vec<McpResource<TemplateResource>>, McpSetupError> {
        resources()
    }
    fn resource_templates() -> Result<Vec<McpResourceTemplate>, McpSetupError> {
        resource_templates()
    }
}

fn described(
    address: TemplateResource,
    title: &str,
    description: &str,
    mime: &str,
) -> Result<McpResource<TemplateResource>, McpSetupError> {
    McpResource::new(address, |uri| {
        Resource::new(uri, title)
            .with_title(title)
            .with_description(description)
            .with_mime_type(mime)
    })
}

fn resources() -> Result<Vec<McpResource<TemplateResource>>, McpSetupError> {
    let mut resources = vec![described(
        TemplateResource::Docs,
        "Server documents",
        "Index of the crate documents embedded at build time.",
        "application/json",
    )?];
    for (document, title) in [
        (TemplateDocument::Agents, "Agent work manual"),
        (TemplateDocument::Design, "Domain design"),
    ] {
        resources.push(described(
            TemplateResource::Document(document),
            title,
            "Crate document embedded at build time.",
            "text/markdown",
        )?);
    }
    resources.push(described(
        TemplateResource::Contract,
        "Contract declaration",
        "Machine-readable contract revision and compliance declaration.",
        "application/json",
    )?);
    resources.push(described(
        TemplateResource::items(),
        "Your items",
        "The first page of up to 100 of your tenant's items in creation order. Read a later \
         page by adding the page's `nextCursor` as `cursor`.",
        "application/json",
    )?);
    Ok(resources)
}

fn resource_templates() -> Result<Vec<McpResourceTemplate>, McpSetupError> {
    [
        (
            "template://docs/{doc_id}",
            "Server document",
            "Embedded crate document body.",
            "text/markdown",
        ),
        (
            "template://items{?cursor}",
            "Your items",
            "A page of your tenant's items in creation order.",
            "application/json",
        ),
        (
            "template://item/{item_id}",
            "Item",
            "One of your tenant's items.",
            "application/json",
        ),
    ]
    .into_iter()
    .map(|(uri, title, description, mime)| {
        let template = ResourceTemplateUri::new(uri).map_err(|_| McpSetupError::InvalidTemplate)?;
        McpResourceTemplate::new(template, |uri| {
            ResourceTemplate::new(uri, title)
                .with_title(title)
                .with_description(description)
                .with_mime_type(mime)
        })
    })
    .collect()
}

fn server_config() -> ServerConfig {
    let mut capabilities = ServerCapabilities::builder()
        .enable_tools()
        .enable_resources()
        .enable_completions()
        .build();
    if let Some(registration) = super::declarations::REGISTRATION.get() {
        veoveo_sdk_rust::registration::mcp::advertise(
            &mut capabilities,
            registration,
            &super::requirements::AUTHORIZATION,
        );
    }
    let mut info = ServerConfig::default();
    info.capabilities = capabilities;
    info.server_info = rmcp::model::Implementation::new("template", env!("CARGO_PKG_VERSION"));
    info.instructions = Some(
        "Keep a list of named items for your tenant. Call `create_item` with a name, read an \
         item at its `resultUri`, and list your items at `template://items`."
            .to_owned(),
    );
    info
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_setup_round_trips_every_resource_and_template() {
        let setup = &*super::SERVER_SETUP;
        assert_eq!(setup.resources().len(), 5);
        assert_eq!(setup.resource_templates().len(), 3);
    }
}
