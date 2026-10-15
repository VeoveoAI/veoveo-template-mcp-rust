//! The server's configuration. Every flag has its environment variable. The Gateway
//! issues the database login and the access-token trust when the server registers, so
//! neither is configured here.
use std::path::PathBuf;

use clap::Parser;
use secrecy::SecretString;
use url::Url;
use veoveo_sdk_rust::{PublicDeployment, parse_allowed_host_authority};

#[derive(Parser)]
#[command(name = "veoveo-template-mcp-rust", about = "Template MCP server")]
pub(super) struct Args {
    #[arg(long, env = "VEOVEO_PORT", default_value_t = 8810)]
    pub port: u16,
    /// The origin where the Gateway reaches the server; it serves MCP at
    /// `{public_base_url}/template/mcp` and registers that URL.
    #[arg(long, env = "VEOVEO_PUBLIC_BASE_URL")]
    pub public_base_url: String,
    /// The Gateway's RFC 8414 issuer; registration discovers its endpoint from there.
    #[arg(long, env = "VEOVEO_GATEWAY_ISSUER")]
    pub gateway_issuer: Url,
    /// The installation's registration secret, or the placeholder the installation's edge
    /// replaces.
    #[arg(
        long,
        env = "VEOVEO_REGISTRATION_SECRET",
        hide_env_values = true,
        value_parser = parse_secret
    )]
    pub registration_secret: SecretString,
    /// Holds the server's registration key, which keeps its namespace across restarts.
    #[arg(
        long,
        env = "VEOVEO_KEY_DIRECTORY",
        default_value = "/var/lib/veoveo/template/keys"
    )]
    pub key_directory: PathBuf,
    #[arg(long, env = "VEOVEO_ALLOW_LOOPBACK_HOSTS", default_value_t = false)]
    pub allow_loopback_hosts: bool,
    #[arg(
        long = "allowed-host",
        env = "VEOVEO_ALLOWED_HOSTS",
        value_name = "HOST",
        value_delimiter = ',',
        value_parser = parse_allowed_host
    )]
    pub allowed_hosts: Vec<String>,
}

impl Args {
    pub fn public_deployment(&self) -> anyhow::Result<PublicDeployment> {
        PublicDeployment::new(&self.public_base_url)
    }
}

fn parse_secret(value: &str) -> Result<SecretString, String> {
    (!value.is_empty())
        .then(|| SecretString::from(value))
        .ok_or_else(|| "secret must not be empty".to_owned())
}

fn parse_allowed_host(value: &str) -> Result<String, String> {
    let value = value.trim();
    parse_allowed_host_authority(value)
        .map(|_| value.to_owned())
        .ok_or_else(|| "expected a host authority such as vv-tpl-mcp-rust.internal:8810".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gateway_and_the_secret_are_required() {
        let base = [
            "veoveo-template-mcp-rust",
            "--public-base-url",
            "http://vv-tpl-mcp-rust.internal:8810",
        ];
        assert!(Args::try_parse_from(base).is_err());
        let args = Args::try_parse_from(base.into_iter().chain([
            "--gateway-issuer",
            "http://vv-gateway.internal:8080",
            "--registration-secret",
            "placeholder",
            "--allowed-host",
            "vv-tpl-mcp-rust.internal:8810",
        ]))
        .unwrap();
        assert_eq!(args.port, 8810);
        assert_eq!(args.allowed_hosts, ["vv-tpl-mcp-rust.internal:8810"]);
        assert!(args.public_deployment().is_ok());
    }
}
