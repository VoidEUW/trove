//! Finds and prepares the data folder (ADR 8). The shell resolves the path and hands it to
//! `trove-store`, which never guesses it.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// The variable that overrides the default location; tests and development use it.
pub const TROVE_HOME: &str = "TROVE_HOME";

/// The data folder on macOS and Linux, inside the user's home directory: `~/.trove`.
pub const UNIX_FOLDER: &str = ".trove";

/// The data folder on Windows, inside `%APPDATA%`: `%APPDATA%\trove`.
pub const WINDOWS_FOLDER: &str = "trove";

/// The variable that holds `%APPDATA%` on Windows.
const APP_DATA: &str = "APPDATA";

/// Why the data folder could not be found or prepared.
#[derive(Debug, thiserror::Error)]
pub enum DataFolderError {
    #[error(
        "cannot find the home directory; set {TROVE_HOME} to choose where Trove keeps its data"
    )]
    NoHomeDir,
    #[error("{TROVE_HOME} must be an absolute path, got `{}`", .0.display())]
    NotAbsolute(PathBuf),
    #[error("cannot create `{}`: {source}", path.display())]
    Create { path: PathBuf, source: io::Error },
}

/// The places a default location is built from, read from the environment once so the rule itself
/// stays a pure function.
#[derive(Clone, Debug, Default)]
pub struct Environment {
    pub trove_home: Option<OsString>,
    pub home_dir: Option<PathBuf>,
    /// `%APPDATA%`, set on Windows.
    pub app_data: Option<PathBuf>,
}

impl Environment {
    pub fn from_process() -> Self {
        Self {
            trove_home: std::env::var_os(TROVE_HOME),
            home_dir: std::env::home_dir(),
            app_data: std::env::var_os(APP_DATA).map(PathBuf::from),
        }
    }
}

/// The root of the data folder and its fixed entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataFolder {
    root: PathBuf,
}

impl DataFolder {
    /// `TROVE_HOME` if set; otherwise `~/.trove` on macOS and Linux and `%APPDATA%\trove` on Windows.
    /// Nothing is created.
    pub fn resolve(env: &Environment, windows: bool) -> Result<Self, DataFolderError> {
        let root = match env.trove_home.as_ref().filter(|value| !value.is_empty()) {
            Some(value) => {
                let path = PathBuf::from(value);
                if !path.is_absolute() {
                    return Err(DataFolderError::NotAbsolute(path));
                }
                path
            }
            None if windows => env
                .app_data
                .as_ref()
                .map(|app_data| app_data.join(WINDOWS_FOLDER))
                .ok_or(DataFolderError::NoHomeDir)?,
            None => env
                .home_dir
                .as_ref()
                .map(|home| home.join(UNIX_FOLDER))
                .ok_or(DataFolderError::NoHomeDir)?,
        };
        Ok(Self { root })
    }

    /// [`DataFolder::resolve`] for the running process.
    pub fn from_process() -> Result<Self, DataFolderError> {
        Self::resolve(&Environment::from_process(), cfg!(windows))
    }

    /// Creates the folder with `files/`, `backups/` and `logs/`. Existing entries are left alone.
    pub fn create(&self) -> Result<(), DataFolderError> {
        for path in [self.root.clone(), self.files(), self.backups(), self.logs()] {
            std::fs::create_dir_all(&path)
                .map_err(|source| DataFolderError::Create { path, source })?;
        }
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn database(&self) -> PathBuf {
        self.root.join(trove_store::DATABASE_FILE)
    }

    pub fn files(&self) -> PathBuf {
        self.root.join("files")
    }

    pub fn backups(&self) -> PathBuf {
        self.root.join(trove_store::BACKUPS_DIR)
    }

    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(trove_home: Option<&Path>) -> Environment {
        Environment {
            trove_home: trove_home.map(|path| path.as_os_str().to_owned()),
            home_dir: Some(PathBuf::from("/Users/anna")),
            app_data: Some(PathBuf::from(r"C:\Users\anna\AppData\Roaming")),
        }
    }

    #[test]
    fn trove_home_wins_over_the_defaults() {
        let home = tempfile::tempdir().unwrap();
        for windows in [false, true] {
            let folder = DataFolder::resolve(&env(Some(home.path())), windows).unwrap();
            assert_eq!(folder.root(), home.path());
        }
    }

    #[test]
    fn macos_and_linux_default_to_dot_trove_in_the_home_directory() {
        let folder = DataFolder::resolve(&env(None), false).unwrap();
        assert_eq!(folder.root(), Path::new("/Users/anna/.trove"));
    }

    #[test]
    fn windows_defaults_to_trove_in_appdata() {
        let folder = DataFolder::resolve(&env(None), true).unwrap();
        assert_eq!(
            folder.root(),
            Path::new(r"C:\Users\anna\AppData\Roaming").join("trove")
        );
    }

    #[test]
    fn an_empty_trove_home_counts_as_unset() {
        let mut environment = env(None);
        environment.trove_home = Some(OsString::new());
        let folder = DataFolder::resolve(&environment, false).unwrap();
        assert_eq!(folder.root(), Path::new("/Users/anna/.trove"));
    }

    #[test]
    fn a_relative_trove_home_is_refused() {
        let result = DataFolder::resolve(&env(Some(Path::new("data"))), false);
        assert!(matches!(result, Err(DataFolderError::NotAbsolute(_))));
    }

    #[test]
    fn without_any_location_resolving_fails_instead_of_guessing() {
        let none = Environment::default();
        assert!(matches!(
            DataFolder::resolve(&none, false),
            Err(DataFolderError::NoHomeDir)
        ));
        assert!(matches!(
            DataFolder::resolve(&none, true),
            Err(DataFolderError::NoHomeDir)
        ));
    }

    #[test]
    fn create_makes_the_folder_with_files_backups_and_logs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("trove-data");
        let folder = DataFolder::resolve(&env(Some(&root)), false).unwrap();
        folder.create().unwrap();
        for entry in ["files", "backups", "logs"] {
            assert!(root.join(entry).is_dir(), "{entry}");
        }
        assert_eq!(folder.database(), root.join("trove.db"));
        assert!(
            !folder.database().exists(),
            "the store creates trove.db, not the shell"
        );
    }

    #[test]
    fn create_keeps_what_is_already_there() {
        let temp = tempfile::tempdir().unwrap();
        let folder = DataFolder::resolve(&env(Some(temp.path())), false).unwrap();
        std::fs::create_dir_all(folder.files()).unwrap();
        std::fs::write(folder.files().join("keep"), b"x").unwrap();
        folder.create().unwrap();
        assert!(folder.files().join("keep").is_file());
    }

    #[test]
    fn create_reports_the_path_it_could_not_make() {
        let temp = tempfile::tempdir().unwrap();
        let blocker = temp.path().join("file");
        std::fs::write(&blocker, b"x").unwrap();
        let folder = DataFolder::resolve(&env(Some(&blocker.join("inside"))), false).unwrap();
        match folder.create() {
            Err(DataFolderError::Create { path, .. }) => assert!(path.starts_with(&blocker)),
            other => panic!("expected a Create error, got {other:?}"),
        }
    }
}
