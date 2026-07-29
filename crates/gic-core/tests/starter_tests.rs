use gic_core::starter_engine::models::{ProjectConfig, TemplateGenerator};
use gic_core::starter_engine::templates::{
    ansible::AnsibleStarter,
    docker::{DockerComposeStarter, DockerStarter},
    github_actions::GithubActionsStarter,
    kubernetes::KubernetesStarter,
    terraform::TerraformStarter,
};

#[test]
fn test_kubernetes_template_generation() {
    let mut config = ProjectConfig::new();
    config.set_answer("k8s_kind".to_string(), "Kubernetes Deployment".to_string());
    config.set_answer("app_name".to_string(), "my-web-app".to_string());
    config.set_answer("container_image".to_string(), "nginx:latest".to_string());

    let starter = KubernetesStarter;
    let files = starter.generate(&config);
    assert!(!files.is_empty());

    let deployment_file = &files[0];
    assert!(deployment_file.content.contains("kind: Deployment"));
    assert!(deployment_file.content.contains("my-web-app"));
    assert!(deployment_file.content.contains("nginx:latest"));
}

#[test]
fn test_docker_template_generation() {
    let mut config = ProjectConfig::new();
    config.set_answer("language".to_string(), "Node".to_string());

    let starter = DockerStarter;
    let files = starter.generate(&config);
    assert_eq!(files.len(), 1);
    assert!(files[0].content.contains("FROM node:18-alpine"));
}

#[test]
fn test_docker_compose_template_generation() {
    let mut config = ProjectConfig::new();
    config.set_answer("stack".to_string(), "Node + PostgreSQL".to_string());

    let starter = DockerComposeStarter;
    let files = starter.generate(&config);
    assert_eq!(files.len(), 1);
    assert!(files[0].content.contains("postgres:14-alpine"));
}

#[test]
fn test_terraform_template_generation() {
    let mut config = ProjectConfig::new();
    config.set_answer("cloud".to_string(), "AWS".to_string());
    config.set_answer("resource".to_string(), "EC2".to_string());

    let starter = TerraformStarter;
    let files = starter.generate(&config);
    assert_eq!(files.len(), 1);
    assert!(files[0].content.contains("aws_instance"));
}

#[test]
fn test_ansible_template_generation() {
    let mut config = ProjectConfig::new();
    config.set_answer("playbook".to_string(), "Docker Install".to_string());

    let starter = AnsibleStarter;
    let files = starter.generate(&config);
    assert_eq!(files.len(), 1);
    assert!(files[0].content.contains("Install Docker"));
}

#[test]
fn test_github_actions_template_generation() {
    let mut config = ProjectConfig::new();
    config.set_answer("workflow".to_string(), "CI".to_string());

    let starter = GithubActionsStarter;
    let files = starter.generate(&config);
    assert_eq!(files.len(), 1);
    assert!(files[0].content.contains("name: CI"));
}
