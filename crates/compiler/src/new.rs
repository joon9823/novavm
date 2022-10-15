use std::{
    fmt::Display,
    fs::create_dir_all,
    io::Write,
    path::{Path, PathBuf},
};

use move_deps::move_package::source_package::layout::SourcePackageLayout;

pub const NOVA_STDLIB_PACKAGE_NAME: &str = "NovaStdlib";
pub const NOVA_STDLIB_PACKAGE_PATH: &str = "{ \
    git = \"https://github.com/Kernel-Labs/novavm.git\", \
    subdir = \"vm/src/nova_stdlib\", rev = \"main\" \
}";
pub const NOVA_STDLIB_ADDR_NAME: &str = "std";
pub const NOVA_STDLIB_ADDR_VALUE: &str = "0x1";

pub struct New {
    /// The name of the package to be created.
    pub name: String,
}

impl New {
    pub fn execute_with_defaults(self, path: Option<PathBuf>) -> anyhow::Result<()> {
        self.execute(
            path,
            "0.0.0",
            [(NOVA_STDLIB_PACKAGE_NAME, NOVA_STDLIB_PACKAGE_PATH)],
            [(NOVA_STDLIB_ADDR_NAME, NOVA_STDLIB_ADDR_VALUE)],
            "",
        )
    }

    pub fn execute(
        self,
        path: Option<PathBuf>,
        version: &str,
        deps: impl IntoIterator<Item = (impl Display, impl Display)>,
        addrs: impl IntoIterator<Item = (impl Display, impl Display)>,
        custom: &str, // anything else that needs to end up being in Move.toml (or empty string)
    ) -> anyhow::Result<()> {
        // TODO warn on build config flags
        let Self { name } = self;
        let p: PathBuf;
        let path: &Path = match path {
            Some(path) => {
                p = path;
                &p
            }
            None => Path::new(&name),
        };
        create_dir_all(path.join(SourcePackageLayout::Sources.path()))?;
        let mut w = std::fs::File::create(path.join(SourcePackageLayout::Manifest.path()))?;
        writeln!(
            &mut w,
            "[package]
name = \"{name}\"
version = \"{version}\"

[dependencies]"
        )?;
        for (dep_name, dep_val) in deps {
            writeln!(w, "{dep_name} = {dep_val}")?;
        }

        writeln!(
            w,
            "
[addresses]"
        )?;
        for (addr_name, addr_val) in addrs {
            writeln!(w, "{addr_name} =  \"{addr_val}\"")?;
        }
        if !custom.is_empty() {
            writeln!(w, "{}", custom)?;
        }
        Ok(())
    }
}
