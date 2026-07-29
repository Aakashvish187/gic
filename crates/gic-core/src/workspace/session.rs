use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSession {
    pub active_pane: usize,
    pub open_files: Vec<PathBuf>,
}

impl WorkspaceSession {
    pub fn save(&self, project_root: &Path) -> std::io::Result<()> {
        let session_dir = project_root.join(".gic");
        if !session_dir.exists() {
            std::fs::create_dir_all(&session_dir)?;
        }
        let session_file = session_dir.join("workspace.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(session_file, content)?;
        Ok(())
    }

    pub fn load(project_root: &Path) -> Option<Self> {
        let session_file = project_root.join(".gic/workspace.json");
        if let Ok(content) = std::fs::read_to_string(session_file) {
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }
}
