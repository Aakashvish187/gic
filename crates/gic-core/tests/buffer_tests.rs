use gic_core::{CursorPosition, TextBuffer};

#[test]
fn test_buffer_insert_and_delete() {
    let mut buffer = TextBuffer::new();
    buffer.insert_str("apiVersion: v1\nkind: Pod\n");
    assert_eq!(buffer.line_count(), 3);
    assert_eq!(buffer.line(0).unwrap(), "apiVersion: v1");
    assert_eq!(buffer.line(1).unwrap(), "kind: Pod");
}

#[test]
fn test_buffer_undo_redo() {
    let mut buffer = TextBuffer::new();
    buffer.insert_str("services:\n  app:\n    image: nginx\n");
    assert_eq!(buffer.line_count(), 4);

    let undo_success = buffer.undo();
    assert!(undo_success.is_ok());

    let redo_success = buffer.redo();
    assert!(redo_success.is_ok());
}

#[test]
fn test_buffer_search_matching() {
    let mut buffer = TextBuffer::new();
    buffer.insert_str("resource \"aws_instance\" \"web\" {\n  ami = \"ami-12345\"\n}\n");

    let query = "aws_instance";
    let mut matches = Vec::new();

    for (row_idx, line) in buffer.lines().iter().enumerate() {
        if let Some(idx) = line.find(query) {
            matches.push(CursorPosition::new(row_idx, idx));
        }
    }

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].row, 0);
    assert_eq!(matches[0].col, 10);
}
