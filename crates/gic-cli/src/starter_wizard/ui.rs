use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gic_core::starter_engine::models::{
    ProjectConfig, ProjectType, Question, QuestionType, TemplateGenerator,
};
use gic_core::starter_engine::templates::kubernetes::KubernetesStarter;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io;

pub fn run_wizard(
    path: Option<&std::path::Path>,
    project_type: ProjectType,
) -> Result<Option<ProjectConfig>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let questions = get_questions_for_type(&project_type);
    let mut config = ProjectConfig::new();

    let mut current_q_idx = 0;

    // UI States
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut text_input = String::new();
    let mut text_cursor = 0;

    let mut finished = false;
    let mut canceled = false;

    // Filter questions based on condition
    let mut active_questions = Vec::new();
    for q in &questions {
        if let Some(cond) = q.condition {
            if cond(&config) {
                active_questions.push(q.clone());
            }
        } else {
            active_questions.push(q.clone());
        }
    }

    while !finished && !canceled && current_q_idx < active_questions.len() {
        let q = &active_questions[current_q_idx];

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(5)].as_ref())
                .split(f.size());

            let title = Paragraph::new(Line::from(vec![Span::styled(
                "GIC Project Starter Engine",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            let content_block = Block::default()
                .borders(Borders::ALL)
                .title(q.prompt.clone());

            match &q.q_type {
                QuestionType::Select(options) => {
                    let items: Vec<ListItem> =
                        options.iter().map(|o| ListItem::new(o.as_str())).collect();

                    let list = List::new(items)
                        .block(content_block)
                        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                        .highlight_symbol(">> ");

                    f.render_stateful_widget(list, chunks[1], &mut list_state);
                }
                QuestionType::Boolean => {
                    let items = vec![ListItem::new("Yes"), ListItem::new("No")];
                    let list = List::new(items)
                        .block(content_block)
                        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                        .highlight_symbol(">> ");

                    f.render_stateful_widget(list, chunks[1], &mut list_state);
                }
                QuestionType::Text { default } => {
                    let display_text = if text_input.is_empty() {
                        format!(
                            "{} (default: {})\n\n[Type your answer and press Enter to submit]",
                            text_input, default
                        )
                    } else {
                        format!("{}\n\n[Press Enter to submit]", text_input)
                    };

                    let p = Paragraph::new(display_text).block(content_block);
                    f.render_widget(p, chunks[1]);
                }
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => {
                        canceled = true;
                    }
                    KeyCode::Up => match &q.q_type {
                        QuestionType::Select(options) => {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        options.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));
                        }
                        QuestionType::Boolean => {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));
                        }
                        _ => {}
                    },
                    KeyCode::Down => match &q.q_type {
                        QuestionType::Select(options) => {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    if i >= options.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));
                        }
                        QuestionType::Boolean => {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    if i >= 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));
                        }
                        _ => {}
                    },
                    KeyCode::Char(c) => {
                        if let QuestionType::Text { .. } = &q.q_type {
                            text_input.push(c);
                            text_cursor += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        if let QuestionType::Text { .. } = &q.q_type {
                            if !text_input.is_empty() {
                                text_input.pop();
                                text_cursor -= 1;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // Record answer
                        match &q.q_type {
                            QuestionType::Select(options) => {
                                let idx = list_state.selected().unwrap_or(0);
                                config.set_answer(q.id.clone(), options[idx].clone());
                            }
                            QuestionType::Boolean => {
                                let idx = list_state.selected().unwrap_or(0);
                                let ans = if idx == 0 { "true" } else { "false" };
                                config.set_answer(q.id.clone(), ans.to_string());
                            }
                            QuestionType::Text { default } => {
                                let ans = if text_input.is_empty() {
                                    default.clone()
                                } else {
                                    text_input.clone()
                                };
                                config.set_answer(q.id.clone(), ans);
                            }
                        }

                        // Move to next question
                        current_q_idx += 1;
                        list_state.select(Some(0));
                        text_input.clear();
                        text_cursor = 0;

                        // Re-evaluate active questions based on new config
                        active_questions.clear();
                        for q in &questions {
                            if let Some(cond) = q.condition {
                                if cond(&config) {
                                    active_questions.push(q.clone());
                                }
                            } else {
                                active_questions.push(q.clone());
                            }
                        }

                        if current_q_idx >= active_questions.len() {
                            finished = true;
                        }
                    }
                    _ => {}
                }
            } // Close if key.kind == ...
        }
    }

    // Teardown terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if canceled {
        Ok(None)
    } else {
        Ok(Some(config))
    }
}

fn get_questions_for_type(project_type: &ProjectType) -> Vec<Question> {
    match project_type {
        ProjectType::DockerCompose => vec![Question {
            id: "stack".to_string(),
            prompt: "Choose Stack".to_string(),
            q_type: QuestionType::Select(vec![
                "Manual (Empty File)".to_string(),
                "Node + PostgreSQL".to_string(),
                "Redis".to_string(),
                "MongoDB".to_string(),
                "WordPress".to_string(),
                "ELK".to_string(),
                "Prometheus + Grafana".to_string(),
                "Nginx".to_string(),
                "Blank".to_string(),
            ]),
            condition: None,
        }],
        ProjectType::Docker => vec![Question {
            id: "language".to_string(),
            prompt: "Choose Language".to_string(),
            q_type: QuestionType::Select(vec![
                "Manual (Empty File)".to_string(),
                "Node".to_string(),
                "Python".to_string(),
                "Java".to_string(),
                "Go".to_string(),
                "Rust".to_string(),
                "PHP".to_string(),
                ".NET".to_string(),
            ]),
            condition: None,
        }],
        ProjectType::Terraform => vec![
            Question {
                id: "cloud".to_string(),
                prompt: "Choose Cloud".to_string(),
                q_type: QuestionType::Select(vec![
                    "Manual (Empty File)".to_string(),
                    "AWS".to_string(),
                    "Azure".to_string(),
                    "GCP".to_string(),
                ]),
                condition: None,
            },
            Question {
                id: "resource".to_string(),
                prompt: "Choose Resource".to_string(),
                q_type: QuestionType::Select(vec![
                    "EC2".to_string(),
                    "VPC".to_string(),
                    "IAM".to_string(),
                    "Lambda".to_string(),
                    "EKS".to_string(),
                    "RDS".to_string(),
                    "S3".to_string(),
                ]),
                condition: Some(|config| {
                    config
                        .get_answer("cloud")
                        .map(|k| k != "Manual (Empty File)")
                        .unwrap_or(true)
                }),
            },
        ],
        ProjectType::Ansible => vec![Question {
            id: "playbook".to_string(),
            prompt: "Choose Playbook".to_string(),
            q_type: QuestionType::Select(vec![
                "Manual (Empty File)".to_string(),
                "Docker Install".to_string(),
                "Nginx Install".to_string(),
                "Create User".to_string(),
                "SSH Hardening".to_string(),
                "Deploy App".to_string(),
            ]),
            condition: None,
        }],
        ProjectType::GithubActions => vec![Question {
            id: "workflow".to_string(),
            prompt: "Choose Workflow".to_string(),
            q_type: QuestionType::Select(vec![
                "Manual (Empty File)".to_string(),
                "CI".to_string(),
                "Docker Build".to_string(),
                "Terraform".to_string(),
                "Deploy Kubernetes".to_string(),
                "Release".to_string(),
            ]),
            condition: None,
        }],
        ProjectType::Kubernetes => KubernetesStarter::get_questions(),
        _ => vec![], // Shouldn't happen if generic bypasses wizard
    }
}
