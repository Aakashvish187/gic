use crate::starter_engine::models::{GeneratedFile, ProjectConfig, TemplateGenerator};

pub struct TerraformStarter;

impl TemplateGenerator for TerraformStarter {
    fn generate(&self, config: &ProjectConfig) -> Vec<GeneratedFile> {
        let cloud = config
            .get_answer("cloud")
            .map(|s| s.as_str())
            .unwrap_or("AWS");
        let resource = config
            .get_answer("resource")
            .map(|s| s.as_str())
            .unwrap_or("EC2");

        let content = match cloud {
            "AWS" => match resource {
                "EC2" => {
                    r#"provider "aws" {
  region = "us-east-1"
}

resource "aws_instance" "app_server" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "t2.micro"

  tags = {
    Name = "ExampleAppServerInstance"
  }
}
"#
                }
                "VPC" => {
                    r#"provider "aws" {
  region = "us-east-1"
}

resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"
  
  tags = {
    Name = "main"
  }
}
"#
                }
                "S3" => {
                    r#"provider "aws" {
  region = "us-east-1"
}

resource "aws_s3_bucket" "b" {
  bucket = "my-tf-test-bucket"

  tags = {
    Name        = "My bucket"
    Environment = "Dev"
  }
}
"#
                }
                _ => {
                    r#"provider "aws" {
  region = "us-east-1"
}
"#
                }
            },
            "Azure" => match resource {
                _ => {
                    r#"provider "azurerm" {
  features {}
}

resource "azurerm_resource_group" "example" {
  name     = "example-resources"
  location = "West Europe"
}
"#
                }
            },
            "GCP" => match resource {
                _ => {
                    r#"provider "google" {
  project = "my-project-id"
  region  = "us-central1"
  zone    = "us-central1-c"
}
"#
                }
            },
            _ => "",
        };

        vec![GeneratedFile {
            path: "main.tf".to_string(),
            content: content.to_string(),
        }]
    }
}
