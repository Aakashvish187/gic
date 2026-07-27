//! Centralized project metadata and About information provider for GIC.

use std::fmt;

/// Project metadata constants.
pub struct ProjectMetadata;

impl ProjectMetadata {
    pub const NAME: &'static str = "GIC";
    pub const FULL_NAME: &'static str = "General Infrastructure Console";
    pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    pub const ORIGINAL_CREATOR: &'static str = "Aakash Vishwakarma";
    pub const CREATOR_ROLE: &'static str = "Founder & Lead Architect";
    pub const PUBLISHER: &'static str = "Aakash Vishwakarma";
    pub const ORGANIZATION: &'static str = "Independent Open Source Project";
    pub const COPYRIGHT: &'static str = "Copyright (c) 2026 Aakash Vishwakarma";
    pub const LICENSE: &'static str = "MIT License";
    pub const WEBSITE: &'static str = "https://github.com/Aakashvish187";
    pub const REPOSITORY: &'static str = "https://github.com/Aakashvish187/gic";
    pub const SUPPORT_EMAIL: &'static str = "aakashvish1920@gmail.com";
}

/// Comprehensive information bundle exposed by the About command/provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutInfo {
    pub name: &'static str,
    pub full_name: &'static str,
    pub version: &'static str,
    pub original_creator: &'static str,
    pub creator_role: &'static str,
    pub publisher: &'static str,
    pub organization: &'static str,
    pub copyright: &'static str,
    pub license: &'static str,
    pub repository: &'static str,
    pub website: &'static str,
    pub support_email: &'static str,
    pub build_date: String,
    pub rust_version: String,
    pub os: &'static str,
    pub arch: &'static str,
}

impl AboutInfo {
    /// Generates a human-readable formatted summary string.
    pub fn summary(&self) -> String {
        format!(
            "{} ({}) v{}\n\
             {}\n\
             Creator:      {} ({})\n\
             Publisher:    {}\n\
             Organization: {}\n\
             License:      {}\n\
             Repository:   {}\n\
             Website:      {}\n\
             Support:      {}\n\
             OS/Arch:      {}/{}\n\
             Rust Version: {}\n\
             Build Date:   {}",
            self.name,
            self.full_name,
            self.version,
            self.copyright,
            self.original_creator,
            self.creator_role,
            self.publisher,
            self.organization,
            self.license,
            self.repository,
            self.website,
            self.support_email,
            self.os,
            self.arch,
            self.rust_version,
            self.build_date
        )
    }
}

impl fmt::Display for AboutInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// About Provider trait for dependency inversion and query capability.
pub trait AboutProvider {
    fn get_about_info(&self) -> AboutInfo;
}

/// Default implementation of the About Provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultAboutProvider;

impl DefaultAboutProvider {
    pub fn new() -> Self {
        Self
    }
}

impl AboutProvider for DefaultAboutProvider {
    fn get_about_info(&self) -> AboutInfo {
        AboutInfo {
            name: ProjectMetadata::NAME,
            full_name: ProjectMetadata::FULL_NAME,
            version: ProjectMetadata::VERSION,
            original_creator: ProjectMetadata::ORIGINAL_CREATOR,
            creator_role: ProjectMetadata::CREATOR_ROLE,
            publisher: ProjectMetadata::PUBLISHER,
            organization: ProjectMetadata::ORGANIZATION,
            copyright: ProjectMetadata::COPYRIGHT,
            license: ProjectMetadata::LICENSE,
            repository: ProjectMetadata::REPOSITORY,
            website: ProjectMetadata::WEBSITE,
            support_email: ProjectMetadata::SUPPORT_EMAIL,
            build_date: option_env!("BUILD_DATE")
                .unwrap_or("2026-07-27")
                .to_string(),
            rust_version: option_env!("RUSTC_VERSION")
                .unwrap_or("1.75.0+")
                .to_string(),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_metadata_constants() {
        assert_eq!(ProjectMetadata::NAME, "GIC");
        assert_eq!(ProjectMetadata::ORIGINAL_CREATOR, "Aakash Vishwakarma");
        assert_eq!(ProjectMetadata::LICENSE, "MIT License");
        assert!(ProjectMetadata::REPOSITORY.contains("Aakashvish187/gic"));
    }

    #[test]
    fn test_default_about_provider() {
        let provider = DefaultAboutProvider::new();
        let info = provider.get_about_info();
        assert_eq!(info.name, "GIC");
        assert_eq!(info.original_creator, "Aakash Vishwakarma");
        assert_eq!(info.os, std::env::consts::OS);
        assert_eq!(info.arch, std::env::consts::ARCH);

        let summary = info.summary();
        assert!(summary.contains("General Infrastructure Console"));
        assert!(summary.contains("Aakash Vishwakarma"));
    }
}
