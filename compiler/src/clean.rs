use std::{path::{PathBuf, Path}, io::ErrorKind};

use anyhow::bail;
use move_deps::move_command_line_common::env::MOVE_HOME;

pub struct Clean {
    pub clean_cache: bool
}

impl Clean {
    pub fn execute(self, path: Option<PathBuf>) -> anyhow::Result<()> {
        let path = match path{
            Some(p) => p,
            None => Path::new(".").to_path_buf(),
        }.join("build");

        let res = std::fs::remove_dir_all(path);
        match res {
            Ok(_) => {
                let move_home = &*MOVE_HOME;
                if self.clean_cache {
                    match std::fs::remove_dir_all(PathBuf::from(move_home)) {
                        Ok(..) => Ok(()),
                        Err(e) => {
                            match e.kind() {
                                ErrorKind::NotFound => Ok(()),
                                _ => bail!("failed to clean cache: {}", e)
                            }
                        }
                    }
                } else {
                    Ok(())
                }
            },
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(()),
                _ => bail!("failed to clean the package: {}", e),
            }
        }
    }
}