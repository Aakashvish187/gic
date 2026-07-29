mod cli;
mod editor_app;
mod starter_wizard;

use anyhow::Result;
use cli::CliOptions;
use editor_app::EditorApp;
use gic_config::ConfigLoader;
use gic_core::{AboutProvider, DefaultAboutProvider};
use gic_logging::init_logging;
use tracing::info;

fn main() -> Result<()> {
    // 1. Parse Command Line Options
    let options = CliOptions::parse();

    // Check for About or Version flags before bootstrapping TUI
    if options.about {
        let about_provider = DefaultAboutProvider::new();
        println!("{}", about_provider.get_about_info());
        return Ok(());
    }

    if options.version {
        let about_provider = DefaultAboutProvider::new();
        let info = about_provider.get_about_info();
        println!("{} v{}", info.name, info.version);
        return Ok(());
    }

    if options.update {
        let about_provider = DefaultAboutProvider::new();
        let info = about_provider.get_about_info();
        let updater = gic_core::updater::Updater::new(&info.version);
        if let Err(e) = updater.perform_update() {
            eprintln!("Update check failed: {}", e);
        }
        return Ok(());
    }

    let config_path = options
        .config_path
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("gic.toml"));

    // 2. Load Configuration
    let config = ConfigLoader::load_from_file(&config_path)?;

    // 3. Initialize Structured Logging
    let _ = init_logging(&config.logging);
    info!(app_name = %config.app_name, "Starting GIC Infrastructure Editor");

    // 4. Project Starter Engine (Wizard)
    let mut final_file_path = options.file_path;

    let mut should_run_wizard = options.template;
    let mut detected_project_type = gic_core::starter_engine::models::ProjectType::Generic;

    if let Some(path) = &final_file_path {
        let is_new = !path.exists() || options.new_file;
        
        if is_new && !options.template {
            // Detect intent
            detected_project_type = gic_core::starter_engine::detector::detect_intent(path);
            if detected_project_type != gic_core::starter_engine::models::ProjectType::Generic {
                should_run_wizard = true;
            } else {
                // Generic file, just create it empty and skip wizard
                let _ = std::fs::File::create(path);
            }
        } else if options.template {
            detected_project_type = gic_core::starter_engine::detector::detect_intent(path);
        }
    } else {
        // No file specified, skip wizard (unless we want a master wizard later)
    }

    if should_run_wizard {
        info!("Launching Project Starter Wizard for {:?}", detected_project_type);
        if let Some(config) = starter_wizard::run_wizard(final_file_path.as_deref(), detected_project_type.clone())? {
            use gic_core::starter_engine::TemplateGenerator;
            use gic_core::starter_engine::models::ProjectType;

            // Check for manual mode early exit based on typical primary config answers
            let is_manual = config.get_answer("cloud").map(|v| v.as_str()) == Some("Manual (Empty File)") ||
                            config.get_answer("stack").map(|v| v.as_str()) == Some("Manual (Empty File)") ||
                            config.get_answer("language").map(|v| v.as_str()) == Some("Manual (Empty File)") ||
                            config.get_answer("playbook").map(|v| v.as_str()) == Some("Manual (Empty File)") ||
                            config.get_answer("workflow").map(|v| v.as_str()) == Some("Manual (Empty File)") ||
                            config.get_answer("k8s_kind").map(|v| v.as_str()) == Some("Manual (Empty File)");

            if !is_manual {
                let generated_files = match detected_project_type {
                    ProjectType::Kubernetes => gic_core::starter_engine::templates::kubernetes::KubernetesStarter.generate(&config),
                    ProjectType::Terraform => gic_core::starter_engine::templates::terraform::TerraformStarter.generate(&config),
                    ProjectType::Docker => gic_core::starter_engine::templates::docker::DockerStarter.generate(&config),
                    ProjectType::DockerCompose => gic_core::starter_engine::templates::docker::DockerComposeStarter.generate(&config),
                    ProjectType::Ansible => gic_core::starter_engine::templates::ansible::AnsibleStarter.generate(&config),
                    ProjectType::GithubActions => gic_core::starter_engine::templates::github_actions::GithubActionsStarter.generate(&config),
                    _ => {
                        // For any unknown type, generate a blank file
                        if let Some(path) = &final_file_path {
                            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            vec![gic_core::starter_engine::models::GeneratedFile {
                                path: filename,
                                content: String::new(),
                            }]
                        } else {
                            vec![]
                        }
                    }
                };

                if !generated_files.is_empty() {
                    let base_dir = std::env::current_dir()?;
                    gic_core::starter_engine::generator::write_generated_files(generated_files.clone(), &base_dir)?;
                    
                    // Override path with the first generated file
                    final_file_path = Some(base_dir.join(&generated_files[0].path));
                }
            } else {
                // Manual mode selected: create the empty file so the editor doesn't complain, 
                // but do not generate boilerplate.
                if let Some(path) = &final_file_path {
                    let _ = std::fs::File::create(path);
                }
            }
        }
    }

    // 5. Launch Interactive Editor Application
    let app = EditorApp::new(final_file_path, config.ui, options.debug);
    app.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_cli_options_integration() {
        let opts = CliOptions::parse_from(vec!["gic", "--config", "test.toml"]);
        assert_eq!(
            opts.config_path,
            Some(std::path::PathBuf::from("test.toml"))
        );
    }
}
