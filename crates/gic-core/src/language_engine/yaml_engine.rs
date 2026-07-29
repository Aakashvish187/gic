//! # YAML / Kubernetes Language Engine
//!
//! Provides diagnostics, completions, and hover documentation for
//! YAML files, with enhanced support for Kubernetes manifests.

use super::{Completion, CompletionKind, EngineDiagnostic, EngineQuickFix, HoverInfo, LanguageEngine};
use super::context::ContextResolver;
use super::schema::LanguageSchema;
use super::kubernetes_schema::KubernetesSchema;

pub struct YamlEngine {
    k8s_schema: KubernetesSchema,
}

impl YamlEngine {
    pub fn new() -> Self {
        Self {
            k8s_schema: KubernetesSchema::new(),
        }
    }

    fn check_yaml_syntax(&self, content: &str) -> Vec<EngineDiagnostic> {
        let mut diagnostics = Vec::new();

        for (row, line) in content.lines().enumerate() {
            // Skip empty lines and comments
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for tabs (YAML doesn't allow tabs for indentation)
            if line.contains('\t') {
                let col = line.find('\t').unwrap_or(0);
                diagnostics.push(
                    EngineDiagnostic::error(row, col, "Tab character not allowed in YAML indentation", "yaml")
                        .with_code("YAML001")
                        .with_fix(EngineQuickFix::new(
                            "Replace tab with spaces",
                            row, col, 1, "  ",
                        ).preferred())
                );
            }

            // Trailing whitespace check removed as it produces annoying grey underlines while typing.

            // Check for key-value pairs that are missing a space after colon
            if trimmed.contains(':') && !trimmed.starts_with('-') && !trimmed.starts_with('#') {
                if let Some(colon_pos) = trimmed.find(':') {
                    let after_colon = &trimmed[colon_pos + 1..];
                    if !after_colon.is_empty()
                        && !after_colon.starts_with(' ')
                        && !after_colon.starts_with('\n')
                        && after_colon != ":"
                    {
                        // But skip URLs like http://
                        if !trimmed[..colon_pos].ends_with("http")
                            && !trimmed[..colon_pos].ends_with("https")
                        {
                            let abs_col = line.find(':').unwrap_or(0);
                            diagnostics.push(
                                EngineDiagnostic::error(row, abs_col + 1, "Missing space after ':'", "yaml")
                                    .with_code("YAML003")
                                    .with_fix(EngineQuickFix::new(
                                        "Add space after ':'",
                                        row, abs_col + 1, 0, " ",
                                    ).preferred())
                            );
                        }
                    }
                }
            }

            // Check for duplicate keys on adjacent lines (simple heuristic)
            // A more robust check would need a full YAML parser

            // Check indentation consistency (must be multiple of 2)
            let indent = line.len() - line.trim_start().len();
            if indent > 0 && indent % 2 != 0 {
                diagnostics.push(
                    EngineDiagnostic::warning(row, 0, "Odd indentation (expected multiple of 2)", "yaml")
                        .with_code("YAML004")
                );
            }
        }

        diagnostics
    }

    fn check_kubernetes(&self, content: &str) -> Vec<EngineDiagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut has_api_version = false;
        let mut has_kind = false;
        let mut has_metadata = false;
        let mut has_metadata_name = false;
        let mut kind_value = String::new();

        for (row, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("apiVersion:") {
                has_api_version = true;
            }
            if trimmed.starts_with("kind:") {
                has_kind = true;
                kind_value = trimmed.trim_start_matches("kind:").trim().to_string();

                // Validate known kinds
                let known_kinds = [
                    "Deployment", "Service", "Pod", "ConfigMap", "Secret",
                    "Ingress", "DaemonSet", "StatefulSet", "CronJob", "Job",
                    "Namespace", "ServiceAccount", "ClusterRole", "ClusterRoleBinding",
                    "Role", "RoleBinding", "PersistentVolume", "PersistentVolumeClaim",
                    "HorizontalPodAutoscaler", "NetworkPolicy", "ResourceQuota",
                    "LimitRange", "ReplicaSet",
                ];
                if !kind_value.is_empty() && !known_kinds.contains(&kind_value.as_str()) {
                    diagnostics.push(
                        EngineDiagnostic::warning(row, trimmed.find(&kind_value).unwrap_or(5), 
                            format!("Unknown Kubernetes resource kind: '{}'", kind_value), "kubernetes")
                            .with_code("K8S002")
                            .with_length(kind_value.len())
                    );
                }
            } else if trimmed.starts_with("kind ") && !trimmed.contains(':') {
                let col = line.find("kind").unwrap_or(0);
                diagnostics.push(
                    EngineDiagnostic::error(row, col + 4, "Missing ':' after 'kind'", "kubernetes")
                        .with_code("K8S006")
                        .with_fix(EngineQuickFix::new("Add ':'", row, col + 4, 0, ":").preferred())
                );
            }
            
            if trimmed.starts_with("metadata:") {
                has_metadata = true;
            }
            if trimmed.starts_with("name:") && has_metadata {
                has_metadata_name = true;
            } else if trimmed.starts_with("name ") && !trimmed.contains(':') && has_metadata {
                let col = line.find("name").unwrap_or(0);
                diagnostics.push(
                    EngineDiagnostic::error(row, col + 4, "Missing ':' after 'name'", "kubernetes")
                        .with_code("K8S007")
                        .with_fix(EngineQuickFix::new("Add ':'", row, col + 4, 0, ":").preferred())
                );
            }

            // Check for 'latest' tag
            if trimmed.starts_with("image:") || trimmed.starts_with("- image:") {
                let image_val = trimmed.split(':').skip(1).collect::<Vec<_>>().join(":");
                let image_val = image_val.trim().trim_matches('"').trim_matches('\'');
                if image_val.ends_with(":latest") || (!image_val.contains(':') && !image_val.is_empty() && !image_val.starts_with('#')) {
                    let col = line.find(image_val).unwrap_or(0);
                    diagnostics.push(
                        EngineDiagnostic::warning(row, col,
                            "Using ':latest' tag or untagged image. Pin to a specific version for reproducibility.",
                            "kubernetes")
                            .with_code("K8S003")
                            .with_length(image_val.len())
                    );
                }
            }

            // Check for missing resource limits and probes in containers
            if trimmed == "containers:" {
                let base_indent = line.len() - line.trim_start().len();
                let insert_indent = base_indent + 4;
                let indent_str = " ".repeat(insert_indent);
                
                let mut found_resources = false;
                let mut found_liveness = false;
                let mut found_readiness = false;
                
                for j in (row + 1)..lines.len().min(row + 30) {
                    let next_line = lines[j];
                    let next = next_line.trim();
                    if next.starts_with("resources:") {
                        found_resources = true;
                    }
                    if next.starts_with("livenessProbe:") {
                        found_liveness = true;
                    }
                    if next.starts_with("readinessProbe:") {
                        found_readiness = true;
                    }
                    let next_indent = next_line.len() - next_line.trim_start().len();
                    if !next.is_empty() && !next.starts_with('-') && !next.starts_with('#')
                        && next_indent <= base_indent
                        && j > row + 1
                    {
                        break;
                    }
                }
                
                if !found_resources {
                    let snippet = format!("\n{}resources:\n{}  requests:\n{}    cpu: \"100m\"\n{}    memory: \"128Mi\"\n{}  limits:\n{}    cpu: \"500m\"\n{}    memory: \"512Mi\"", 
                        indent_str, indent_str, indent_str, indent_str, indent_str, indent_str, indent_str);
                    diagnostics.push(
                        EngineDiagnostic::hint(row, 0,
                            "Container spec missing 'resources:' (limits/requests). Set resource limits for production.",
                            "kubernetes")
                            .with_code("K8S004")
                            .with_fix(EngineQuickFix::new("Add resource limits", row, line.len(), 0, snippet).preferred())
                    );
                }
                
                if !found_liveness {
                    let snippet = format!("\n{}livenessProbe:\n{}  httpGet:\n{}    path: █\n{}    port: █\n{}  initialDelaySeconds: 30\n{}  periodSeconds: 10\n{}  timeoutSeconds: 1\n{}  successThreshold: 1\n{}  failureThreshold: 3", 
                        indent_str, indent_str, indent_str, indent_str, indent_str, indent_str, indent_str, indent_str, indent_str);
                    diagnostics.push(
                        EngineDiagnostic::hint(row, 0,
                            "Container spec missing 'livenessProbe'.",
                            "kubernetes")
                            .with_code("K8S008")
                            .with_fix(EngineQuickFix::new("Add livenessProbe", row, line.len(), 0, snippet))
                    );
                }
                
                if !found_readiness {
                    let snippet = format!("\n{}readinessProbe:\n{}  httpGet:\n{}    path: █\n{}    port: █\n{}  initialDelaySeconds: 5\n{}  periodSeconds: 5\n{}  timeoutSeconds: 1\n{}  successThreshold: 1\n{}  failureThreshold: 3", 
                        indent_str, indent_str, indent_str, indent_str, indent_str, indent_str, indent_str, indent_str, indent_str);
                    diagnostics.push(
                        EngineDiagnostic::hint(row, 0,
                            "Container spec missing 'readinessProbe'.",
                            "kubernetes")
                            .with_code("K8S009")
                            .with_fix(EngineQuickFix::new("Add readinessProbe", row, line.len(), 0, snippet))
                    );
                }
            }

            // Check for replicas: with non-integer values
            if trimmed.starts_with("replicas:") {
                let val = trimmed.trim_start_matches("replicas:").trim();
                if !val.is_empty() && val.parse::<u32>().is_err() {
                    let col = line.find(val).unwrap_or(0);
                    let mut diag = EngineDiagnostic::error(row, col,
                            format!("'replicas' expects an integer, got '{}'", val),
                            "kubernetes")
                            .with_code("K8S005")
                            .with_length(val.len());
                            
                    let unquoted = val.trim_matches('"').trim_matches('\'');
                    if unquoted.parse::<u32>().is_ok() {
                        diag = diag.with_fix(EngineQuickFix::new(
                            "Unquote integer", row, col, val.len(), unquoted
                        ).preferred());
                    }
                    
                    diagnostics.push(diag);
                }
            }
        }

        // Top-level missing fields
        if !has_api_version {
            diagnostics.push(
                EngineDiagnostic::error(0, 0, "Missing required field 'apiVersion'", "kubernetes")
                    .with_code("K8S001")
            );
        }
        if !has_kind {
            diagnostics.push(
                EngineDiagnostic::error(0, 0, "Missing required field 'kind'", "kubernetes")
                    .with_code("K8S001")
            );
        }
        if !has_metadata {
            diagnostics.push(
                EngineDiagnostic::warning(0, 0, "Missing 'metadata' section", "kubernetes")
                    .with_code("K8S001")
            );
        }

        diagnostics
    }

    fn kubernetes_completions(&self, content: &str, row: usize, col: usize) -> Vec<Completion> {
        let lines: Vec<&str> = content.lines().collect();
        let current_line = lines.get(row).map(|l| l.trim_start()).unwrap_or("");
        let indent = lines.get(row).map(|l| l.len() - l.trim_start().len()).unwrap_or(0);
        
        let ctx = ContextResolver::resolve_yaml(content, row, col, "kubernetes");
        let mut completions = Vec::new();

        // Top-level completions (indent 0)
        if ctx.path.is_empty() && !current_line.contains(':') {
            if current_line.is_empty() || current_line.starts_with('k') {
                completions.push(Completion::new("kind", "kind: ", CompletionKind::Property).with_detail("Resource kind"));
            }
            if current_line.is_empty() || current_line.starts_with('a') {
                completions.push(Completion::new("apiVersion", "apiVersion: ", CompletionKind::Property).with_detail("API version"));
            }
            if current_line.is_empty() || current_line.starts_with('m') {
                completions.push(Completion::new("metadata", "metadata:\n  name: ", CompletionKind::Snippet).with_detail("Metadata section"));
            }
            if current_line.is_empty() || current_line.starts_with('s') {
                completions.push(Completion::new("spec", "spec:\n  ", CompletionKind::Snippet).with_detail("Spec section"));
            }
        }

        // If typing a value (after colon)
        if current_line.contains(':') {
            let parts: Vec<&str> = current_line.splitn(2, ':').collect();
            let key = parts[0].trim();
            let needs_space = current_line.ends_with(':');
            
            if key == "kind" {
                let kinds = [
                    ("Deployment", "Manages ReplicaSets and Pods"),
                    ("Service", "Exposes Pods as a network service"),
                    ("Pod", "Smallest deployable unit"),
                    ("ConfigMap", "Non-confidential configuration data"),
                    ("Secret", "Confidential data store"),
                    ("Ingress", "HTTP/HTTPS routing rules"),
                ];
                for (kind, detail) in &kinds {
                    let insert = if needs_space { format!(" {}", kind) } else { kind.to_string() };
                    completions.push(Completion::new(*kind, insert, CompletionKind::Type).with_detail(*detail));
                }
            } else if key == "apiVersion" {
                let versions = [
                    ("apps/v1", "Deployments, StatefulSets"),
                    ("v1", "Core resources"),
                ];
                for (ver, detail) in &versions {
                    let insert = if needs_space { format!(" {}", ver) } else { ver.to_string() };
                    completions.push(Completion::new(*ver, insert, CompletionKind::Value).with_detail(*detail));
                }
            } else {
                // Fetch value completions from schema based on context!
                let mut schema_comps = self.k8s_schema.value_completions(&ctx.path, key, ctx.resource_kind.as_deref());
                if needs_space {
                    for comp in &mut schema_comps {
                        comp.insert_text = format!(" {}", comp.insert_text);
                    }
                }
                completions.extend(schema_comps);
            }
        } else {
            // Fetch key completions from schema based on context!
            completions.extend(self.k8s_schema.key_completions(&ctx.path, indent, ctx.resource_kind.as_deref()));
        }

        completions
    }

    fn kubernetes_hover(&self, content: &str, row: usize, col: usize) -> Option<HoverInfo> {
        let lines: Vec<&str> = content.lines().collect();
        let line = lines.get(row)?;
        let trimmed = line.trim();

        // Extract the key name under the cursor
        let key = if let Some(colon_pos) = trimmed.find(':') {
            let key_part = trimmed[..colon_pos].trim();
            // Check if cursor is roughly on the key
            let key_start = line.find(key_part).unwrap_or(0);
            if col >= key_start && col <= key_start + key_part.len() {
                key_part
            } else {
                return None;
            }
        } else {
            return None;
        };

        // Static hovers for top level
        match key {
            "apiVersion" => return Some(HoverInfo::new(
                "apiVersion",
                "Specifies the Kubernetes API version for this resource.")
                .with_syntax("apiVersion: apps/v1")),
            "kind" => return Some(HoverInfo::new(
                "kind",
                "Specifies the type of Kubernetes resource (Deployment, Service, Pod, etc.).")
                .with_syntax("kind: Deployment")),
            "metadata" => return Some(HoverInfo::new(
                "metadata",
                "Data that helps uniquely identify the object. Includes name, namespace, labels, and annotations.")),
            _ => {
                // Fetch from schema
                let ctx = ContextResolver::resolve_yaml(content, row, col, "kubernetes");
                self.k8s_schema.hover_info(&ctx.path, key, ctx.resource_kind.as_deref())
            }
        }
    }
}

impl LanguageEngine for YamlEngine {
    fn name(&self) -> &'static str { "YAML / Kubernetes" }
    fn id(&self) -> &'static str { "yaml" }

    fn diagnostics(&self, content: &str) -> Vec<EngineDiagnostic> {
        let mut diags = self.check_yaml_syntax(content);

        // If it looks like a Kubernetes manifest, run K8s checks too
        let has_api = content.lines().any(|l| l.trim().starts_with("apiVersion:"));
        let has_kind = content.lines().any(|l| l.trim().starts_with("kind:"));
        if has_api || has_kind {
            diags.extend(self.check_kubernetes(content));
        }

        // Sort by row, then severity
        diags.sort_by(|a, b| a.row.cmp(&b.row).then(a.col.cmp(&b.col)));
        diags
    }

    fn completions(&self, content: &str, row: usize, col: usize) -> Vec<Completion> {
        self.kubernetes_completions(content, row, col)
    }

    fn hover(&self, content: &str, row: usize, col: usize) -> Option<HoverInfo> {
        self.kubernetes_hover(content, row, col)
    }

    fn format(&self, content: &str) -> Option<String> {
        // A simple, safe YAML formatter.
        // It avoids parsing to prevent dropping comments.
        let mut formatted = String::with_capacity(content.len());
        for line in content.lines() {
            // 1. Replace tabs with 2 spaces
            let mut line = line.replace("\t", "  ");
            
            // 2. Remove trailing whitespace
            line = line.trim_end().to_string();

            // 3. Ensure a space after colon, but only if we can be reasonably sure it's a key
            // This is a naive heuristic: if the line matches `^ *[\w-]+: \S` but missing the space.
            if let Some(colon_idx) = line.find(':') {
                if colon_idx < line.len() - 1 {
                    let next_char = line.as_bytes()[colon_idx + 1] as char;
                    if next_char != ' ' && next_char != '\n' && next_char != '\r' {
                        let prefix = &line[..colon_idx];
                        // If prefix has no spaces inside, it's likely a key
                        if !prefix.trim().contains(' ') && !prefix.contains('"') && !prefix.contains('\'') {
                            line.insert(colon_idx + 1, ' ');
                        }
                    }
                }
            }

            formatted.push_str(&line);
            formatted.push('\n');
        }
        
        // Remove trailing newline if original didn't have it (or just leave it, most formatters ensure trailing newline)
        if !content.ends_with('\n') && formatted.ends_with('\n') {
            formatted.pop();
        }

        if formatted != content {
            Some(formatted)
        } else {
            None
        }
    }

    fn smart_enter(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("apiVersion:") {
            Some("kind: ".to_string())
        } else if trimmed.starts_with("kind:") {
            Some("metadata:\n  name: ".to_string())
        } else {
            None
        }
    }

    fn template_expansion(&self, keyword: &str) -> Option<String> {
        match keyword.to_lowercase().as_str() {
            "deployment" => Some("apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: █\nspec:\n  replicas: 1\n  selector:\n    matchLabels:\n      app: █\n  template:\n    metadata:\n      labels:\n        app: █\n    spec:\n      containers:\n      - name: █\n        image: █\n        ports:\n        - containerPort: 80".to_string()),
            "service" => Some("apiVersion: v1\nkind: Service\nmetadata:\n  name: █\nspec:\n  selector:\n    app: █\n  ports:\n    - protocol: TCP\n      port: 80\n      targetPort: 80".to_string()),
            "ingress" => Some("apiVersion: networking.k8s.io/v1\nkind: Ingress\nmetadata:\n  name: █\nspec:\n  rules:\n  - host: █\n    http:\n      paths:\n      - path: /\n        pathType: Prefix\n        backend:\n          service:\n            name: █\n            port:\n              number: 80".to_string()),
            "pod" => Some("apiVersion: v1\nkind: Pod\nmetadata:\n  name: █\nspec:\n  containers:\n  - name: █\n    image: █".to_string()),
            _ => None,
        }
    }
}
