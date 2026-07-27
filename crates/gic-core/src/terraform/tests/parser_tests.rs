//! Unit tests for HCL / Terraform parser.

use crate::terraform::parser::TerraformParser;

#[test]
fn test_parse_terraform_hcl_blocks() {
    let source = r#"
terraform {
  required_version = ">= 1.5.0"
  backend "s3" {
    bucket = "my-tf-state"
    key    = "prod/terraform.tfstate"
    region = "us-east-1"
    encrypt = true
  }
}

provider "aws" {
  region = "us-east-1"
}

resource "aws_s3_bucket" "b" {
  bucket = "my-tf-test-bucket"
  acl    = "private"

  tags = {
    Environment = "Dev"
  }
}
"#;

    let parser = TerraformParser::new();
    let ast = parser.parse(source).unwrap();

    assert_eq!(ast.blocks.len(), 3);
    assert_eq!(ast.blocks[0].block_type, "terraform");
    assert_eq!(ast.blocks[1].block_type, "provider");
    assert_eq!(ast.blocks[1].first_label(), Some("aws"));
    assert_eq!(ast.blocks[2].block_type, "resource");
    assert_eq!(ast.blocks[2].first_label(), Some("aws_s3_bucket"));
    assert_eq!(ast.blocks[2].second_label(), Some("b"));
}
