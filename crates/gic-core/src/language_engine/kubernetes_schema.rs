use std::collections::HashMap;
use crate::language_engine::schema::{LanguageSchema, SchemaDataType, SchemaNode, SchemaProperty};

pub struct KubernetesSchema {
    nodes: HashMap<String, SchemaNode>,
}

impl KubernetesSchema {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();

        // --- Container ---
        let container = SchemaNode::new("Container", "A single application container")
            .with_property(
                SchemaProperty::new("name", "Name of the container specified as a DNS_LABEL.", SchemaDataType::String)
                .required()
            )
            .with_property(
                SchemaProperty::new("image", "Container image name.", SchemaDataType::Suggestions(vec!["nginx".to_string(), "redis".to_string(), "postgres".to_string(), "mysql".to_string(), "mongo".to_string(), "rabbitmq".to_string(), "busybox".to_string(), "alpine".to_string(), "ubuntu".to_string(), "node".to_string(), "python".to_string(), "golang".to_string(), "dotnet".to_string(), "java".to_string()]))
                .with_example("nginx:1.24.0-alpine")
                .with_production_recommendation("Pin to a specific version or digest rather than using 'latest'.")
                .with_security_recommendation("Use distroless or alpine base images to reduce attack surface.")
                .with_common_mistakes("Using 'latest' tag can lead to unpredictable deployments.")
            )
            .with_property(
                SchemaProperty::new("imagePullPolicy", "Image pull policy.", SchemaDataType::Enum(vec!["Always".to_string(), "IfNotPresent".to_string(), "Never".to_string()]))
                .with_default("IfNotPresent")
            )
            .with_property(
                SchemaProperty::new("ports", "List of ports to expose from the container.", SchemaDataType::Array(Box::new(SchemaDataType::Object)))
                .with_snippet("ports:\n  - containerPort: █\n    protocol: TCP")
            )
            .with_property(
                SchemaProperty::new("resources", "Compute Resources required by this container.", SchemaDataType::Object)
                .with_snippet("resources:\n  requests:\n    cpu: \"100m\"\n    memory: \"128Mi\"\n  limits:\n    cpu: \"500m\"\n    memory: \"512Mi\"")
                .with_production_recommendation("Always set resource limits and requests to prevent noisy neighbors and OOM kills.")
            )
            .with_property(
                SchemaProperty::new("livenessProbe", "Periodic probe of container liveness.", SchemaDataType::Object)
                .with_snippet("livenessProbe:\n  httpGet:\n    path: █\n    port: █\n  initialDelaySeconds: 30\n  periodSeconds: 10\n  timeoutSeconds: 1\n  successThreshold: 1\n  failureThreshold: 3")
                .with_production_recommendation("Essential for high availability. Use this to detect application deadlocks.")
            )
            .with_property(
                SchemaProperty::new("readinessProbe", "Periodic probe of container readiness.", SchemaDataType::Object)
                .with_snippet("readinessProbe:\n  httpGet:\n    path: █\n    port: █\n  initialDelaySeconds: 5\n  periodSeconds: 5\n  timeoutSeconds: 1\n  successThreshold: 1\n  failureThreshold: 3")
                .with_production_recommendation("Essential to ensure no traffic is routed to the pod until it's ready.")
            )
            .with_property(
                SchemaProperty::new("securityContext", "Security options the container should be run with.", SchemaDataType::Object)
                .with_snippet("securityContext:\n  allowPrivilegeEscalation: false\n  readOnlyRootFilesystem: true\n  runAsNonRoot: true\n  runAsUser: 1000")
                .with_security_recommendation("Drop ALL capabilities, avoid root, and make filesystem read-only.")
            )
            .with_property(
                SchemaProperty::new("env", "List of environment variables to set in the container.", SchemaDataType::Array(Box::new(SchemaDataType::Object)))
                .with_snippet("env:\n  - name: █\n    value: █")
                .with_security_recommendation("Use SecretKeyRef instead of plain text for sensitive data.")
            )
            .with_property(
                SchemaProperty::new("volumeMounts", "Pod volumes to mount into the container's filesystem.", SchemaDataType::Array(Box::new(SchemaDataType::Object)))
                .with_snippet("volumeMounts:\n  - name: █\n    mountPath: █")
            );
        nodes.insert("Container".to_string(), container);

        // --- PodSpec ---
        let pod_spec = SchemaNode::new("PodSpec", "Specification of the desired behavior of the pod.")
            .with_property(
                SchemaProperty::new("containers", "List of containers belonging to the pod.", SchemaDataType::Array(Box::new(SchemaDataType::Object)))
                .required()
                .with_snippet("containers:\n  - name: █\n    image: █\n    imagePullPolicy: IfNotPresent\n    ports:\n      - containerPort: 80\n    resources:\n      requests:\n        cpu: \"100m\"\n        memory: \"128Mi\"\n      limits:\n        cpu: \"500m\"\n        memory: \"512Mi\"")
            )
            .with_property(
                SchemaProperty::new("restartPolicy", "Restart policy for all containers within the pod.", SchemaDataType::Enum(vec!["Always".to_string(), "OnFailure".to_string(), "Never".to_string()]))
                .with_default("Always")
            )
            .with_property(
                SchemaProperty::new("volumes", "List of volumes that can be mounted by containers belonging to the pod.", SchemaDataType::Array(Box::new(SchemaDataType::Object)))
                .with_snippet("volumes:\n  - name: █\n    emptyDir: {}")
            );
        nodes.insert("PodSpec".to_string(), pod_spec);

        // --- DeploymentSpec ---
        let deployment_spec = SchemaNode::new("DeploymentSpec", "Specification of the desired behavior of the Deployment.")
            .with_property(
                SchemaProperty::new("replicas", "Number of desired pods.", SchemaDataType::Number)
                .with_default("1")
                .with_production_recommendation("Should be at least 2 for high availability.")
            )
            .with_property(
                SchemaProperty::new("selector", "Label selector for pods.", SchemaDataType::Object)
                .required()
                .with_snippet("selector:\n  matchLabels:\n    app: my-app\n")
            )
            .with_property(
                SchemaProperty::new("template", "Template describes the pods that will be created.", SchemaDataType::Object)
                .required()
                .with_snippet("template:\n  metadata:\n    labels:\n      app: my-app\n  spec:\n    containers:\n      - name: app\n        image: nginx\n")
            );
        nodes.insert("DeploymentSpec".to_string(), deployment_spec);

        Self { nodes }
    }
}

impl LanguageSchema for KubernetesSchema {
    fn resolve_path(&self, path: &[String], resource_kind: Option<&str>) -> Option<&SchemaNode> {
        if path.is_empty() {
            match resource_kind {
                Some("Deployment") | Some("ReplicaSet") | Some("DaemonSet") | Some("StatefulSet") => return self.nodes.get("DeploymentSpec"),
                Some("Pod") => return self.nodes.get("PodSpec"),
                _ => return None,
            }
        }

        let last = path.last().unwrap();
        match last.as_str() {
            "containers" | "initContainers" => self.nodes.get("Container"),
            "spec" => {
                // If it's a Deployment spec vs Pod spec
                if path.len() == 1 {
                    if let Some(kind) = resource_kind {
                        if kind == "Pod" {
                            return self.nodes.get("PodSpec");
                        }
                    }
                    // Assume Deployment spec for top-level spec if unknown or Deployment
                    self.nodes.get("DeploymentSpec")
                } else {
                    // Assume PodSpec if nested (e.g. template.spec)
                    self.nodes.get("PodSpec")
                }
            }
            "template" => {
                // Template contains metadata and spec, but we'll let it just pass through or we could define a PodTemplateSpec
                None
            }
            _ => {
                // Heuristic based on the parent path
                if path.contains(&"containers".to_string()) {
                    self.nodes.get("Container")
                } else {
                    None
                }
            }
        }
    }
}
