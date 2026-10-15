//! The one table that says which permission each of the server's targets requires. It
//! builds the `ai.veoveo/authorization` declaration the Gateway evaluates, and every
//! handler checks the same declaration against the permissions the Gateway granted.
use std::sync::LazyLock;

use rmcp::{ErrorData as McpError, RoleServer, service::RequestContext};
use veoveo_sdk_rust::{
    GatewayInternalIdentity, LocalToolName,
    hosting::gateway_identity,
    registration::{
        AuthorizationBuilder, AuthorizationDeclaration, DeclarationError, RequirementTarget,
    },
    server_contract::McpServerContract,
};
use veoveo_types::{PermissionName, ResourceTemplateUri, ResourceUri};

use super::setup::TemplateContract;
use crate::contract::TemplatePermission::{self, *};

/// Every tool and the permission it requires.
pub(crate) const TOOLS: &[(&str, TemplatePermission)] = &[("create_item", ItemsWrite)];

/// Every fixed resource.
pub(crate) const RESOURCES: &[(&str, TemplatePermission)] = &[
    ("template://docs", ItemsRead),
    ("template://contract", ItemsRead),
    ("template://items", ItemsRead),
];

/// Every parameterized resource, as simple templates that match the concrete URIs the
/// server reads.
pub(crate) const RESOURCE_TEMPLATES: &[(&str, TemplatePermission)] = &[
    ("template://docs/{doc_id}", ItemsRead),
    ("template://items?{query}", ItemsRead),
    ("template://item/{item_id}", ItemsRead),
];

/// The server's authorization declaration, checked against its namespace as the Gateway
/// checks it.
pub static AUTHORIZATION: LazyLock<AuthorizationDeclaration> = LazyLock::new(|| {
    declaration().expect("the template's requirement table forms a valid declaration")
});

pub(crate) fn declaration() -> Result<AuthorizationDeclaration, DeclarationError> {
    let invalid = |field: &'static str| DeclarationError::Invalid {
        field,
        reason: "invalid template target spelling",
    };
    let mut builder = AuthorizationBuilder::<TemplatePermission>::new();
    for (tool, permission) in TOOLS {
        builder = builder.tool(
            LocalToolName::parse(*tool).map_err(|_| invalid("tool"))?,
            *permission,
        );
    }
    for (uri, permission) in RESOURCES {
        builder = builder.resource(
            ResourceUri::new(*uri).map_err(|_| invalid("resource"))?,
            *permission,
        );
    }
    for (template, permission) in RESOURCE_TEMPLATES {
        builder = builder.resource_template(
            ResourceTemplateUri::new(*template).map_err(|_| invalid("resource template"))?,
            *permission,
        );
    }
    builder.build(&TemplateContract::slug(), &TemplateContract::scheme())
}

/// A request target whose required permission comes from the declaration.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Target<'a> {
    Tool(&'a str),
    Resource(&'a str),
}

/// The verified Gateway identity, when it grants the permission `target` requires.
pub(crate) fn require(
    context: &RequestContext<RoleServer>,
    target: Target<'_>,
) -> Result<GatewayInternalIdentity, McpError> {
    let identity = gateway_identity(context)?;
    let permission = required(target)?;
    identity.grants(&permission).then_some(identity).ok_or_else(|| {
        McpError::invalid_request(
            format!(
                "You don't have permission to make this request. Missing permission `{permission}`."
            ),
            None,
        )
    })
}

/// The permission `target` requires. A target the declaration omits is denied.
pub(crate) fn required(target: Target<'_>) -> Result<PermissionName, McpError> {
    let undeclared = || {
        McpError::invalid_request(
            "This request names no target the server declares a permission for.".to_owned(),
            None,
        )
    };
    let declared = match target {
        Target::Tool(tool) => {
            let tool = LocalToolName::parse(tool).map_err(|_| undeclared())?;
            AUTHORIZATION
                .required(RequirementTarget::Tool(&tool))
                .cloned()
        }
        Target::Resource(uri) => {
            let uri = ResourceUri::new(uri).map_err(|_| undeclared())?;
            AUTHORIZATION
                .required(RequirementTarget::Resource(&uri))
                .cloned()
        }
    };
    declared
        .map(|declared| declared.permission)
        .ok_or_else(undeclared)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn the_declaration_declares_every_permission() {
        let declared: BTreeSet<String> = declaration()
            .unwrap()
            .permissions
            .iter()
            .map(|permission| permission.permission.as_str().to_owned())
            .collect();
        let expected: BTreeSet<String> = TemplatePermission::ALL
            .iter()
            .map(|permission| permission.as_str().to_owned())
            .collect();
        assert_eq!(declared, expected);
    }

    #[test]
    fn every_tool_the_server_lists_has_exactly_one_requirement() {
        let listed: BTreeSet<String> = super::super::TemplateMcp::tools()
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();
        let required: BTreeSet<String> = TOOLS.iter().map(|(tool, _)| (*tool).to_owned()).collect();
        assert_eq!(listed, required);
    }

    #[test]
    fn concrete_addresses_resolve_and_undeclared_targets_are_denied() {
        for concrete in [
            "template://docs/agents",
            "template://items",
            "template://items?cursor=abc",
            "template://item/0190f3a2-7b1c-7d2e-8f3a-4b5c6d7e8f90",
        ] {
            assert_eq!(
                required(Target::Resource(concrete)).unwrap(),
                ItemsRead.name(),
                "{concrete}"
            );
        }
        assert_eq!(
            required(Target::Tool("create_item")).unwrap(),
            ItemsWrite.name()
        );
        assert!(required(Target::Tool("delete_item")).is_err());
        assert!(required(Target::Resource("template://other")).is_err());
    }

    #[test]
    fn tool_annotations_agree_with_the_declaration() {
        let tools = super::super::TemplateMcp::tools().list_all();
        veoveo_sdk_rust::registration::mcp::check_tool_annotations(&AUTHORIZATION, &tools).unwrap();
    }
}
