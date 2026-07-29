use std::path::PathBuf;

/// Command line arguments parsed for GIC CLI binary.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CliOptions {
    pub config_path: Option<PathBuf>,
    pub file_path: Option<PathBuf>,
    pub about: bool,
    pub version: bool,
    pub update: bool,
    pub debug: bool,
    pub template: bool,
    pub new_file: bool,
}

impl CliOptions {
    /// Parses environment arguments into `CliOptions`.
    pub fn parse() -> Self {
        Self::parse_from(std::env::args())
    }

    /// Helper for parsing arguments from an iterator (enabling clean unit testing).
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let mut config_path = None;
        let mut file_path = None;
        let mut about = false;
        let mut version = false;
        let mut update = false;
        let mut debug = false;
        let mut template = false;
        let mut new_file = false;
        let mut iter = args.into_iter().map(Into::into);

        // Skip binary name
        let _ = iter.next();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-c" | "--config" => {
                    if let Some(path) = iter.next() {
                        config_path = Some(PathBuf::from(path));
                    }
                }
                "-a" | "--about" | "about" => {
                    about = true;
                }
                "-v" | "--version" | "version" => {
                    version = true;
                }
                "-u" | "--update" | "update" => {
                    update = true;
                }
                "--debug" => {
                    debug = true;
                }
                "-t" | "--template" => {
                    template = true;
                }
                "-n" | "--new" => {
                    new_file = true;
                }
                other => {
                    if !other.starts_with('-') && file_path.is_none() {
                        file_path = Some(PathBuf::from(other));
                    }
                }
            }
        }

        Self {
            config_path,
            file_path,
            about,
            version,
            update,
            debug,
            template,
            new_file,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_args() {
        let args = vec!["gic"];
        let opts = CliOptions::parse_from(args);
        assert_eq!(opts.config_path, None);
        assert!(!opts.about);
        assert!(!opts.version);
    }

    #[test]
    fn test_parse_config_flag_long() {
        let args = vec!["gic", "--config", "/etc/gic/config.toml"];
        let opts = CliOptions::parse_from(args);
        assert_eq!(
            opts.config_path,
            Some(PathBuf::from("/etc/gic/config.toml"))
        );
    }

    #[test]
    fn test_parse_config_flag_short() {
        let args = vec!["gic", "-c", "custom.toml"];
        let opts = CliOptions::parse_from(args);
        assert_eq!(opts.config_path, Some(PathBuf::from("custom.toml")));
    }

    #[test]
    fn test_parse_about_flag() {
        let args = vec!["gic", "--about"];
        let opts = CliOptions::parse_from(args);
        assert!(opts.about);
    }

    #[test]
    fn test_parse_version_flag() {
        let args = vec!["gic", "--version"];
        let opts = CliOptions::parse_from(args);
        assert!(opts.version);
    }

    #[test]
    fn test_parse_file_path() {
        let args = vec!["gic", "file.yaml"];
        let opts = CliOptions::parse_from(args);
        assert_eq!(opts.file_path, Some(PathBuf::from("file.yaml")));
    }
}
