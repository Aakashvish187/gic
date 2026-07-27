//! Unit tests for Command Registry.

use crate::linux::commands::CommandRegistry;
use crate::linux::shell::BashParser;

#[test]
fn test_unknown_command_detection() {
    let source = "mkdir /tmp/dir\nnot_a_real_command --help";
    let parser = BashParser::new();
    let ast = parser.parse(source).unwrap();

    let registry = CommandRegistry::new();
    let diags = registry.validate_commands(&ast).unwrap();

    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "lin-cmd-unknown");
    assert!(diags[0].message.contains("not_a_real_command"));
}
