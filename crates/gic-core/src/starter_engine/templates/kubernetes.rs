use crate::starter_engine::models::{GeneratedFile, ProjectConfig, Question, QuestionType, TemplateGenerator};

pub struct KubernetesStarter;

impl KubernetesStarter {
    pub fn get_questions() -> Vec<Question> {
        vec![
            Question {
                id: "k8s_kind".to_string(),
                prompt: "What would you like to deploy?".to_string(),
                q_type: QuestionType::Select(vec![
                    "Kubernetes Deployment".to_string(),
                    "Kubernetes Service".to_string(),
                    "Ingress".to_string(),
                    "ConfigMap".to_string(),
                    "Secret".to_string(),
                    "StatefulSet".to_string(),
                    "DaemonSet".to_string(),
                    "Job".to_string(),
                    "CronJob".to_string(),
                    "Manual (Empty File)".to_string(),
                ]),
                condition: None,
            },
            Question {
                id: "k8s_app".to_string(),
                prompt: "What application are you deploying?".to_string(),
                q_type: QuestionType::Select(vec![
                    "Nginx Website".to_string(),
                    "Node.js API".to_string(),
                    "FastAPI".to_string(),
                    "Spring Boot".to_string(),
                    "Redis".to_string(),
                    "PostgreSQL".to_string(),
                    "Custom Image".to_string(),
                    "Blank Deployment".to_string(),
                ]),
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "app_name".to_string(),
                prompt: "Application Name:".to_string(),
                q_type: QuestionType::Text { default: "my-app".to_string() },
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k != "Manual (Empty File)").unwrap_or(true)
                }),
            },
            Question {
                id: "namespace".to_string(),
                prompt: "Namespace:".to_string(),
                q_type: QuestionType::Text { default: "default".to_string() },
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k != "Manual (Empty File)").unwrap_or(true)
                }),
            },
            Question {
                id: "replicas".to_string(),
                prompt: "Number of replicas:".to_string(),
                q_type: QuestionType::Text { default: "3".to_string() },
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "container_port".to_string(),
                prompt: "Container Port:".to_string(),
                q_type: QuestionType::Text { default: "80".to_string() },
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "image".to_string(),
                prompt: "Container Image:".to_string(),
                q_type: QuestionType::Text { default: "nginx:latest".to_string() },
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "resource_requests".to_string(),
                prompt: "Would you like Resource Requests?".to_string(),
                q_type: QuestionType::Boolean,
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "resource_limits".to_string(),
                prompt: "Would you like Resource Limits?".to_string(),
                q_type: QuestionType::Boolean,
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "liveness_probe".to_string(),
                prompt: "Enable Liveness Probe?".to_string(),
                q_type: QuestionType::Boolean,
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "readiness_probe".to_string(),
                prompt: "Enable Readiness Probe?".to_string(),
                q_type: QuestionType::Boolean,
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "enable_service".to_string(),
                prompt: "Generate Service?".to_string(),
                q_type: QuestionType::Boolean,
                condition: Some(|config| {
                    config.get_answer("k8s_kind").map(|k| k == "Kubernetes Deployment").unwrap_or(false)
                }),
            },
            Question {
                id: "enable_ingress".to_string(),
                prompt: "Generate Ingress?".to_string(),
                q_type: QuestionType::Boolean,
                condition: Some(|config| {
                    let kind = config.get_answer("k8s_kind").map(|k| k.as_str()).unwrap_or("");
                    let has_service = config.get_answer("enable_service").map(|v| v == "true").unwrap_or(false);
                    (kind == "Kubernetes Deployment" && has_service) || kind == "Kubernetes Service"
                }),
            },
        ]
    }

    fn generate_deployment(config: &ProjectConfig) -> GeneratedFile {
        let app_name = config.get_answer("app_name").unwrap_or(&"my-app".to_string()).clone();
        let namespace = config.get_answer("namespace").unwrap_or(&"default".to_string()).clone();
        let replicas = config.get_answer("replicas").unwrap_or(&"3".to_string()).clone();
        let image = config.get_answer("image").unwrap_or(&"nginx:latest".to_string()).clone();
        let port = config.get_answer("container_port").unwrap_or(&"80".to_string()).clone();

        let reqs = if config.get_answer("resource_requests").map(|v| v == "true").unwrap_or(false) {
            "          requests:\n            cpu: \"100m\"\n            memory: \"128Mi\"\n"
        } else { "" };

        let limits = if config.get_answer("resource_limits").map(|v| v == "true").unwrap_or(false) {
            "          limits:\n            cpu: \"500m\"\n            memory: \"512Mi\"\n"
        } else { "" };

        let resources_block = if !reqs.is_empty() || !limits.is_empty() {
            format!("        resources:\n{}{}", reqs, limits)
        } else {
            String::new()
        };

        let liveness = if config.get_answer("liveness_probe").map(|v| v == "true").unwrap_or(false) {
            format!("        livenessProbe:\n          httpGet:\n            path: /\n            port: {}\n          initialDelaySeconds: 15\n          periodSeconds: 20\n", port)
        } else { String::new() };

        let readiness = if config.get_answer("readiness_probe").map(|v| v == "true").unwrap_or(false) {
            format!("        readinessProbe:\n          httpGet:\n            path: /\n            port: {}\n          initialDelaySeconds: 5\n          periodSeconds: 10\n", port)
        } else { String::new() };

        let content = format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {app_name}
  namespace: {namespace}
  labels:
    app: {app_name}
spec:
  replicas: {replicas}
  selector:
    matchLabels:
      app: {app_name}
  template:
    metadata:
      labels:
        app: {app_name}
    spec:
      containers:
      - name: {app_name}
        image: {image}
        ports:
        - containerPort: {port}
{resources_block}{liveness}{readiness}"#
        );

        GeneratedFile {
            path: format!("k8s/deployment.yaml"),
            content,
        }
    }

    fn generate_service(config: &ProjectConfig) -> GeneratedFile {
        let app_name = config.get_answer("app_name").unwrap_or(&"my-app".to_string()).clone();
        let namespace = config.get_answer("namespace").unwrap_or(&"default".to_string()).clone();
        let port = config.get_answer("container_port").unwrap_or(&"80".to_string()).clone();

        let content = format!(
            r#"apiVersion: v1
kind: Service
metadata:
  name: {app_name}
  namespace: {namespace}
  labels:
    app: {app_name}
spec:
  selector:
    app: {app_name}
  ports:
  - port: 80
    targetPort: {port}
  type: ClusterIP"#
        );

        GeneratedFile {
            path: format!("k8s/service.yaml"),
            content,
        }
    }

    fn generate_ingress(config: &ProjectConfig) -> GeneratedFile {
        let app_name = config.get_answer("app_name").unwrap_or(&"my-app".to_string()).clone();
        let namespace = config.get_answer("namespace").unwrap_or(&"default".to_string()).clone();

        let content = format!(
            r#"apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {app_name}
  namespace: {namespace}
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: {app_name}.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: {app_name}
            port:
              number: 80"#
        );

        GeneratedFile {
            path: format!("k8s/ingress.yaml"),
            content,
        }
    }

    fn generate_readme(config: &ProjectConfig) -> GeneratedFile {
        let app_name = config.get_answer("app_name").unwrap_or(&"my-app".to_string()).clone();

        let content = format!(
            r#"# {app_name} Infrastructure

Generated by GIC Project Starter Engine.

## Deployment

To deploy this project to your cluster, run:
```bash
kubectl apply -f k8s/
```
"#
        );

        GeneratedFile {
            path: format!("k8s/README.md"),
            content,
        }
    }
}

impl TemplateGenerator for KubernetesStarter {
    fn generate(&self, config: &ProjectConfig) -> Vec<GeneratedFile> {
        let mut files = Vec::new();
        
        let kind = config.get_answer("k8s_kind").map(|k| k.as_str()).unwrap_or("Kubernetes Deployment");

        if kind == "Kubernetes Deployment" {
            files.push(Self::generate_deployment(config));
            
            if config.get_answer("enable_service").map(|v| v == "true").unwrap_or(false) {
                files.push(Self::generate_service(config));
            }
            if config.get_answer("enable_ingress").map(|v| v == "true").unwrap_or(false) {
                files.push(Self::generate_ingress(config));
            }
        } else if kind == "Kubernetes Service" {
            files.push(Self::generate_service(config));
            if config.get_answer("enable_ingress").map(|v| v == "true").unwrap_or(false) {
                files.push(Self::generate_ingress(config));
            }
        } else if kind == "Ingress" {
            files.push(Self::generate_ingress(config));
        } else if kind == "Manual (Empty File)" {
            // Return nothing so the editor just opens a blank file
            return files;
        }

        files.push(Self::generate_readme(config));

        files
    }
}
