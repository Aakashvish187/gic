use crate::starter_engine::models::{GeneratedFile, ProjectConfig, TemplateGenerator};

pub struct GithubActionsStarter;

impl TemplateGenerator for GithubActionsStarter {
    fn generate(&self, config: &ProjectConfig) -> Vec<GeneratedFile> {
        let workflow = config
            .get_answer("workflow")
            .map(|s| s.as_str())
            .unwrap_or("CI");

        let content = match workflow {
            "CI" => {
                r#"name: CI

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run a one-line script
        run: echo Hello, world!

      - name: Run a multi-line script
        run: |
          echo Add other actions to build,
          echo test, and deploy your project.
"#
            }
            "Docker Build" => {
                r#"name: Docker Build and Push

on:
  push:
    branches: [ "main" ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Login to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: user/app:latest
"#
            }
            "Terraform" => {
                r#"name: 'Terraform'

on:
  push:
    branches: [ "main" ]
  pull_request:

jobs:
  terraform:
    name: 'Terraform'
    runs-on: ubuntu-latest

    steps:
    - name: Checkout
      uses: actions/checkout@v3

    - name: Setup Terraform
      uses: hashicorp/setup-terraform@v2

    - name: Terraform Init
      run: terraform init

    - name: Terraform Format
      run: terraform fmt -check

    - name: Terraform Plan
      run: terraform plan -input=false

    - name: Terraform Apply
      if: github.ref == 'refs/heads/main' && github.event_name == 'push'
      run: terraform apply -auto-approve -input=false
"#
            }
            "Deploy Kubernetes" => {
                r#"name: Deploy to Kubernetes

on:
  push:
    branches: [ "main" ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3

    - name: Set up kubectl
      uses: azure/setup-kubectl@v3

    - name: Set Kubeconfig
      run: |
        mkdir -p $HOME/.kube
        echo "${{ secrets.KUBECONFIG }}" > $HOME/.kube/config

    - name: Deploy
      run: kubectl apply -f k8s/
"#
            }
            "Release" => {
                r#"name: Create Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          generate_release_notes: true
"#
            }
            _ => {
                r#"name: Blank Workflow

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Hello World"
"#
            }
        };

        vec![GeneratedFile {
            path: ".github/workflows/workflow.yml".to_string(),
            content: content.to_string(),
        }]
    }
}
