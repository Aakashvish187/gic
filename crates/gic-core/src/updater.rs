use std::fmt;

#[derive(Debug)]
pub enum UpdateError {
    NetworkError(String),
    AlreadyLatest(String),
    InstallationFailed(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateError::NetworkError(msg) => write!(f, "Network error during update check: {}", msg),
            UpdateError::AlreadyLatest(ver) => write!(f, "GIC is already up to date (version {})", ver),
            UpdateError::InstallationFailed(msg) => write!(f, "Failed to install update: {}", msg),
        }
    }
}

pub struct Updater {
    pub current_version: String,
    pub repo: String,
}

impl Updater {
    pub fn new(current_version: &str) -> Self {
        Self {
            current_version: current_version.to_string(),
            repo: "Aakashvish187/gic".to_string(),
        }
    }

    pub fn check_for_updates(&self) -> Result<String, UpdateError> {
        println!("Checking GitHub Releases ({}) for updates...", self.repo);
        
        let status = self_update::backends::github::Update::configure()
            .repo_owner("Aakashvish187")
            .repo_name("gic")
            .bin_name("gic")
            .show_download_progress(true)
            .current_version(&self.current_version)
            .build()
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let latest = status.get_latest_release().map_err(|e| UpdateError::NetworkError(e.to_string()))?;
        let latest_ver = latest.version;
        
        if self.current_version == latest_ver {
            Err(UpdateError::AlreadyLatest(self.current_version.clone()))
        } else {
            Ok(latest_ver)
        }
    }

    pub fn perform_update(&self) -> Result<(), UpdateError> {
        println!("Current GIC version: v{}", self.current_version);
        
        // This actually checks AND updates if a newer version is available.
        let status = self_update::backends::github::Update::configure()
            .repo_owner("Aakashvish187")
            .repo_name("gic")
            .bin_name("gic")
            .show_download_progress(true)
            .current_version(&self.current_version)
            .build()
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?
            .update()
            .map_err(|e| UpdateError::InstallationFailed(e.to_string()))?;

        if status.updated() {
            println!("Successfully updated GIC to v{}!", status.version());
            Ok(())
        } else {
            println!("You are running the latest version of GIC (v{}). No update needed.", self.current_version);
            Ok(())
        }
    }
}
