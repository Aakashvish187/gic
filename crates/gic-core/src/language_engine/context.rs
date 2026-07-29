use std::fmt::Display;

/// Represents the logical position of the cursor within a structured document (like YAML or JSON).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorContext {
    /// The resolved document type (e.g., "kubernetes").
    pub document_type: String,
    /// The hierarchical path of keys from the root to the cursor (e.g., ["spec", "template", "spec", "containers"]).
    pub path: Vec<String>,
    /// The indentation level at the cursor position (number of spaces).
    pub indent: usize,
    /// True if the cursor is within an array element (e.g., after a `- `).
    pub in_array: bool,
    /// The string the user is currently typing, if any.
    pub current_word: String,
    /// The detected resource kind (e.g., "Deployment"), if applicable.
    pub resource_kind: Option<String>,
}

impl CursorContext {
    /// Returns the path as a dot-separated string (e.g., "spec.template.spec.containers").
    pub fn path_string(&self) -> String {
        self.path.join(".")
    }

    /// Returns the immediate parent key, if any.
    pub fn parent_key(&self) -> Option<&String> {
        self.path.last()
    }
}

impl Display for CursorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context: {}", self.path_string())
    }
}

/// A lightweight parser that determines the `CursorContext` based on indentation and syntax.
pub struct ContextResolver;

impl ContextResolver {
    /// Resolves the context for a YAML document at the given row and col.
    pub fn resolve_yaml(content: &str, row: usize, col: usize, doc_type: &str) -> CursorContext {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() || row >= lines.len() {
            return CursorContext {
                document_type: doc_type.to_string(),
                path: Vec::new(),
                indent: 0,
                in_array: false,
                current_word: String::new(),
                resource_kind: None,
            };
        }

        let mut resource_kind = None;
        if doc_type == "kubernetes" {
            for line in &lines {
                let t = line.trim();
                if t.starts_with("kind:") {
                    let k = t.trim_start_matches("kind:").trim();
                    if !k.is_empty() {
                        resource_kind = Some(k.to_string());
                        break;
                    }
                }
            }
        }

        let mut current_indent = col;
        let mut path = Vec::new();
        let mut in_array = false;

        let current_line = lines[row];
        let trimmed_line = current_line.trim_start();
        let actual_indent = current_line.len() - trimmed_line.len();
        
        // Use the actual indent if the cursor is past it, otherwise use the cursor col
        let mut logical_indent = if col >= actual_indent { actual_indent } else { col };

        // Determine if we are typing an array item
        if trimmed_line.starts_with("- ") {
            in_array = true;
            if logical_indent == actual_indent {
                logical_indent += 2;
            }
        }

        // Walk backwards to build the path based on indentation
        let mut target_indent = logical_indent;

        for (i, line) in lines.iter().take(row).enumerate().rev() {
            let t = line.trim_start();
            if t.is_empty() || t.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            let line_indent = line.len() - t.len();
            
            // Calculate the actual indentation of the key
            let key_indent = if t.starts_with("- ") {
                line_indent + 2
            } else {
                line_indent
            };

            // Check if this line is a parent (less indented)
            if key_indent < target_indent {
                // It must be a key-value pair to be a parent
                if let Some(colon_idx) = t.find(':') {
                    // Make sure it's not a value containing a colon
                    if colon_idx == t.len() - 1 || t[colon_idx + 1..].starts_with(' ') {
                        // Extract the key
                        let mut key = t[..colon_idx].trim().to_string();
                        if key.starts_with("- ") {
                            key = key[2..].trim().to_string();
                        }
                        path.push(key);
                        target_indent = key_indent;
                    }
                }
            }
        }

        // Reverse the path so it goes from root to leaf
        path.reverse();

        // Find current word
        let line_up_to_cursor = &current_line[..col.min(current_line.len())];
        let current_word = line_up_to_cursor
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .last()
            .unwrap_or("")
            .to_string();

        CursorContext {
            document_type: doc_type.to_string(),
            path,
            indent: logical_indent,
            in_array,
            current_word,
            resource_kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_context_resolution() {
        let yaml = "
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: nginx
        image: nginx:latest
        ports:
        - containerPort: 80
          ";

        // Let's resolve the context at the end of the last line (row 13, under ports)
        let ctx = ContextResolver::resolve_yaml(yaml, 13, 10, "kubernetes");
        assert_eq!(ctx.path, vec!["spec", "template", "spec", "containers", "ports"]);
    }
}
