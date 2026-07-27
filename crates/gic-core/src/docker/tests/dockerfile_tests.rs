//! Unit tests for Dockerfile Parsing and AST Generation.

use crate::docker::dockerfile::DockerfileParser;
use crate::docker::instructions::InstructionKind;

#[test]
fn test_parse_multi_stage_dockerfile() {
    let source = r#"
FROM golang:1.22-bookworm AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o server .

FROM alpine:3.19
WORKDIR /root/
COPY --from=builder /app/server .
USER 10001
CMD ["./server"]
"#;

    let parser = DockerfileParser::new();
    let ast = parser.parse(source).unwrap();

    assert_eq!(ast.instructions.len(), 11);

    // Verify Stage 1 FROM
    if let InstructionKind::From {
        ref image,
        ref stage_alias,
        ..
    } = ast.instructions[0].kind
    {
        assert_eq!(image, "golang:1.22-bookworm");
        assert_eq!(stage_alias.as_deref(), Some("builder"));
    } else {
        panic!("Expected FROM instruction");
    }

    // Verify Stage 2 COPY --from=builder (at index 8)
    if let InstructionKind::Copy {
        ref from_stage,
        ref destination,
        ..
    } = ast.instructions[8].kind
    {
        assert_eq!(from_stage.as_deref(), Some("builder"));
        assert_eq!(destination, ".");
    } else {
        panic!("Expected COPY --from instruction");
    }
}
