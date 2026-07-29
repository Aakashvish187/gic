use std::path::Path;
use gic_core::starter_engine::detector::detect_intent;
use gic_core::starter_engine::models::ProjectType;

#[test]
fn test_detect_intent_kubernetes() {
    assert_eq!(detect_intent(Path::new("deployment.yaml")), ProjectType::Kubernetes);
    assert_eq!(detect_intent(Path::new("k8s/service.yml")), ProjectType::Kubernetes);
}

#[test]
fn test_detect_intent_docker() {
    assert_eq!(detect_intent(Path::new("Dockerfile")), ProjectType::Docker);
    assert_eq!(detect_intent(Path::new("Dockerfile.dev")), ProjectType::Docker);
}

#[test]
fn test_detect_intent_docker_compose() {
    assert_eq!(detect_intent(Path::new("docker-compose.yml")), ProjectType::DockerCompose);
    assert_eq!(detect_intent(Path::new("docker-compose.override.yaml")), ProjectType::DockerCompose);
}

#[test]
fn test_detect_intent_terraform() {
    assert_eq!(detect_intent(Path::new("main.tf")), ProjectType::Terraform);
    assert_eq!(detect_intent(Path::new("variables.tf")), ProjectType::Terraform);
}

#[test]
fn test_detect_intent_ansible() {
    assert_eq!(detect_intent(Path::new("playbook.yml")), ProjectType::Ansible);
    assert_eq!(detect_intent(Path::new("site.yaml")), ProjectType::Ansible);
}

#[test]
fn test_detect_intent_github_actions() {
    assert_eq!(detect_intent(Path::new(".github/workflows/ci.yml")), ProjectType::GithubActions);
}
