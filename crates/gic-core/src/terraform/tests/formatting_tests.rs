//! Unit tests for Terraform Code Formatter.

use crate::terraform::formatter::TerraformFormatter;

#[test]
fn test_format_terraform_code_indentation() {
    let source = r#"resource "aws_s3_bucket" "b" {
bucket = "my-bucket"
acl = "private"
}"#;

    let formatter = TerraformFormatter::new();
    let formatted = formatter.format(source);

    let expected =
        "resource \"aws_s3_bucket\" \"b\" {\n  bucket = \"my-bucket\"\n  acl = \"private\"\n}\n";
    assert_eq!(formatted, expected);
}
