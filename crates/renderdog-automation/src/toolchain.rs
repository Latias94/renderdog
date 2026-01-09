use std::{
    env,
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RenderDocInstallation {
    pub root_dir: PathBuf,
    pub qrenderdoc_exe: PathBuf,
    pub renderdoccmd_exe: PathBuf,
}

#[derive(Debug, Error)]
pub enum DetectInstallationError {
    #[error("renderdoc installation not found; set RENDERDOG_RENDERDOC_DIR or install RenderDoc")]
    NotFound,
    #[error("renderdoc installation at {0} is missing {1}")]
    MissingComponent(PathBuf, &'static str),
}

impl RenderDocInstallation {
    pub fn detect() -> Result<Self, DetectInstallationError> {
        if let Some(candidate) = env::var_os("RENDERDOG_RENDERDOC_DIR").map(PathBuf::from) {
            return Self::from_root_dir(candidate);
        }

        // Windows default install path.
        #[cfg(windows)]
        {
            let program_files = env::var_os("ProgramFiles").map(PathBuf::from);
            if let Some(pf) = program_files {
                let candidate = pf.join("RenderDoc");
                if candidate.is_dir()
                    && let Ok(install) = Self::from_root_dir(candidate)
                {
                    return Ok(install);
                }
            }
        }

        if let Some(install) = Self::from_path_env() {
            return Ok(install);
        }

        Err(DetectInstallationError::NotFound)
    }

    pub fn from_root_dir(root_dir: PathBuf) -> Result<Self, DetectInstallationError> {
        let qrenderdoc_exe = root_dir.join(Self::qrenderdoc_exe_name());
        let renderdoccmd_exe = root_dir.join(Self::renderdoccmd_exe_name());

        if !qrenderdoc_exe.is_file() {
            return Err(DetectInstallationError::MissingComponent(
                root_dir,
                Self::qrenderdoc_exe_name(),
            ));
        }
        if !renderdoccmd_exe.is_file() {
            return Err(DetectInstallationError::MissingComponent(
                root_dir,
                Self::renderdoccmd_exe_name(),
            ));
        }

        Ok(Self {
            root_dir,
            qrenderdoc_exe,
            renderdoccmd_exe,
        })
    }

    fn qrenderdoc_exe_name() -> &'static str {
        #[cfg(windows)]
        {
            "qrenderdoc.exe"
        }
        #[cfg(not(windows))]
        {
            "qrenderdoc"
        }
    }

    fn renderdoccmd_exe_name() -> &'static str {
        #[cfg(windows)]
        {
            "renderdoccmd.exe"
        }
        #[cfg(not(windows))]
        {
            "renderdoccmd"
        }
    }

    fn from_path_env() -> Option<Self> {
        let qrenderdoc = find_in_path(Self::qrenderdoc_exe_name())?;
        let renderdoccmd = find_in_path(Self::renderdoccmd_exe_name())?;

        let root_dir = qrenderdoc.parent().map(Path::to_path_buf)?;

        Some(Self {
            root_dir,
            qrenderdoc_exe: qrenderdoc,
            renderdoccmd_exe: renderdoccmd,
        })
    }
}

pub fn default_artifacts_dir(cwd: &Path) -> PathBuf {
    cwd.join("artifacts").join("renderdoc")
}

pub fn default_scripts_dir(cwd: &Path) -> PathBuf {
    cwd.join("artifacts").join("renderdoc").join("scripts")
}

pub fn default_exports_dir(cwd: &Path) -> PathBuf {
    cwd.join("artifacts").join("renderdoc").join("exports")
}

fn find_in_path(exe_name: &str) -> Option<PathBuf> {
    let path_env = env::var_os("PATH")?;
    for dir in env::split_paths(&path_env) {
        let candidate = dir.join(exe_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
