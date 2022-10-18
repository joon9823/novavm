use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::bail;
use move_deps::{move_command_line_common::env::MOVE_HOME, move_package::BuildConfig};

pub struct Clean {
    pub clean_cache: bool,
}

impl Clean {
    pub fn execute(self, path: Option<PathBuf>, config: BuildConfig) -> anyhow::Result<()> {
        let path = match path {
            Some(p) => p,
            None => Path::new(".").to_path_buf(),
        };

        if !(path.join("Move.toml").is_file()) {
            bail!("move package not found in {}", path.to_string_lossy())
        }

        let install_dir = match config.install_dir {
            Some(id) => id.to_path_buf(),
            None => PathBuf::from("build"),
        };
        let package_path = path.join(install_dir);

        let res = std::fs::remove_dir_all(package_path);
        match res {
            Ok(_) => {
                let move_home = &*MOVE_HOME;
                if self.clean_cache {
                    match std::fs::remove_dir_all(PathBuf::from(move_home)) {
                        Ok(..) => Ok(()),
                        Err(e) => match e.kind() {
                            ErrorKind::NotFound => Ok(()),
                            _ => bail!("failed to clean cache: {}", e),
                        },
                    }
                } else {
                    Ok(())
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(()),
                _ => bail!("failed to clean the package: {}", e),
            },
        }
    }
}
