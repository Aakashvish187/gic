use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    DockerCompose,
    Docker,
    Kubernetes,
    Helm,
    Terraform,
    Ansible,
    GithubActions,
    Generic,
}


#[derive(Debug, Clone, PartialEq)]
pub enum QuestionType {
    Select(Vec<String>),
    Boolean,
    Text { default: String },
}

#[derive(Debug, Clone)]
pub struct Question {
    pub id: String,
    pub prompt: String,
    pub q_type: QuestionType,
    pub condition: Option<fn(&ProjectConfig) -> bool>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectConfig {
    pub answers: HashMap<String, String>,
}

impl ProjectConfig {
    pub fn new() -> Self {
        Self {
            answers: HashMap::new(),
        }
    }

    pub fn get_answer(&self, id: &str) -> Option<&String> {
        self.answers.get(id)
    }

    pub fn set_answer(&mut self, id: String, answer: String) {
        self.answers.insert(id, answer);
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

pub trait TemplateGenerator {
    fn generate(&self, config: &ProjectConfig) -> Vec<GeneratedFile>;
}
