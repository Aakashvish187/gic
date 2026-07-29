use super::models::ProjectType;
use std::path::Path;

pub fn detect_intent(path: &Path) -> ProjectType {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let file_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Docker and Docker Compose checks
    if file_name.starts_with("docker-compose") || file_name.starts_with("compose.") {
        return ProjectType::DockerCompose;
    }

    if file_name.starts_with("Dockerfile") {
        return ProjectType::Docker;
    }

    // Special exact matches
    match file_name {
        "Chart.yaml" | "values.yaml" => {
            return ProjectType::Helm;
        }
        "playbook.yml" | "playbook.yaml" | "inventory.yml" | "inventory.yaml" | "site.yml" | "site.yaml" => {
            return ProjectType::Ansible;
        }
        _ => {}
    }

    // Path based logic for GitHub Actions
    if let Some(parent) = path.parent() {
        if parent.ends_with(".github/workflows") || parent.ends_with(".github\\workflows") {
            return ProjectType::GithubActions;
        }
    }

    // Extensions
    match file_ext {
        "tf" | "tfvars" => {
            return ProjectType::Terraform;
        }
        "yaml" | "yml" => {
            // Check Kubernetes specific names
            match file_name {
                "deployment.yaml" | "deployment.yml" |
                "service.yaml" | "service.yml" |
                "ingress.yaml" | "ingress.yml" |
                "pod.yaml" | "pod.yml" |
                "configmap.yaml" | "configmap.yml" |
                "secret.yaml" | "secret.yml" => {
                    return ProjectType::Kubernetes;
                }
                _ => return ProjectType::Kubernetes, // Default YAML to Kubernetes if inside IaC context
            }
        }
        _ => return ProjectType::Generic,
    }
}
