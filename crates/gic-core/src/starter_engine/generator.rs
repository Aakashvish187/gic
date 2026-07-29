use crate::starter_engine::models::GeneratedFile;
use std::fs;
use std::path::Path;

pub fn write_generated_files(files: Vec<GeneratedFile>, base_dir: &Path) -> std::io::Result<()> {
    for file in files {
        let full_path = base_dir.join(&file.path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(full_path, file.content)?;
    }
    
    Ok(())
}
