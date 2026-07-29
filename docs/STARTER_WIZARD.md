# Starter Wizard Engine Guide

GIC automatically detects when a target file does not exist, prompts you to create it, and launches an interactive wizard to generate production-ready boilerplate configurations.

## Supported Technologies

1. **Kubernetes (`*.yaml`, `*.yml`)**:
   - Generates Deployment, Service, Ingress, ConfigMap, Secret, StatefulSet, and DaemonSet templates.
2. **Docker (`Dockerfile`)**:
   - Generates multi-stage Dockerfiles for Node.js, Python, Java, Go, Rust, and PHP.
3. **Docker Compose (`docker-compose.yml`)**:
   - Stack options: Node + Postgres, Redis, MongoDB, WordPress, ELK, Prometheus + Grafana.
4. **Terraform (`*.tf`)**:
   - AWS (EC2, VPC, S3), Azure, and GCP boilerplate generation.
5. **Ansible (`playbook.yml`)**:
   - Docker install, Nginx setup, user creation, SSH hardening playbooks.
6. **GitHub Actions (`.github/workflows/*.yml`)**:
   - CI, Docker build & push, Terraform, Kubernetes deploy workflows.
