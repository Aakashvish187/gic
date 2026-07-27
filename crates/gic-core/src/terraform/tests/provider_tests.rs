//! Unit tests for Terraform Provider configuration extraction and validation.

use crate::terraform::parser::TerraformParser;
use crate::terraform::providers::{KnownProvider, ProviderValidator};

#[test]
fn test_extract_provider_configuration() {
    let source = r#"
provider "aws" {
  region  = "us-west-2"
  alias   = "west"
  version = "~> 5.0"
}
"#;

    let parser = TerraformParser::new();
    let ast = parser.parse(source).unwrap();
    let validator = ProviderValidator::new();

    let config = validator.extract_provider_config(&ast.blocks[0]).unwrap();
    assert_eq!(config.name, "aws");
    assert_eq!(config.provider_kind, KnownProvider::AWS);
    assert_eq!(config.alias.as_deref(), Some("west"));
    assert_eq!(config.region.as_deref(), Some("us-west-2"));
    assert_eq!(config.version.as_deref(), Some("~> 5.0"));
}
