//! Unit tests for Terraform Module calls and source classification.

use crate::terraform::modules::{ModuleSourceKind, ModuleValidator};
use crate::terraform::parser::TerraformParser;

#[test]
fn test_module_source_classification() {
    let source = r#"
module "local_vpc" {
  source = "./modules/vpc"
}

module "registry_vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "5.1.0"
}
"#;

    let parser = TerraformParser::new();
    let ast = parser.parse(source).unwrap();
    let validator = ModuleValidator::new();

    let mod1 = validator.extract_module(&ast.blocks[0]).unwrap();
    assert_eq!(mod1.name, "local_vpc");
    assert_eq!(mod1.source_kind, ModuleSourceKind::Local);

    let mod2 = validator.extract_module(&ast.blocks[1]).unwrap();
    assert_eq!(mod2.name, "registry_vpc");
    assert_eq!(mod2.source_kind, ModuleSourceKind::Registry);
    assert_eq!(mod2.version_constraint.as_deref(), Some("5.1.0"));
}
