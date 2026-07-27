//! Unit tests for Bash and POSIX shell parsing.

use crate::linux::shell::{BashParser, ShellKind};

#[test]
fn test_shebang_parsing() {
    let source = "#!/bin/bash\necho 'hello'";
    let parser = BashParser::new();
    let ast = parser.parse(source).unwrap();

    assert!(ast.shebang.is_some());
    assert_eq!(ast.shebang.unwrap().shell, ShellKind::Bash);
    assert_eq!(ast.commands.len(), 1);
    assert_eq!(ast.commands[0].command_name, "echo");
}

#[test]
fn test_pipeline_and_redirection_parsing() {
    let source = "cat /var/log/syslog | grep error > errors.log";
    let parser = BashParser::new();
    let ast = parser.parse(source).unwrap();

    assert_eq!(ast.commands.len(), 1);
    let cmd = &ast.commands[0];
    assert_eq!(cmd.command_name, "cat");
    assert!(cmd.is_piped);
    assert!(cmd.has_redirection);
    assert_eq!(cmd.pipeline_commands.len(), 2);
}
