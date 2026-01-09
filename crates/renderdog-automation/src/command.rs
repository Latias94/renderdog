use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: Option<PathBuf>,
}

impl CommandSpec {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn display_command_line(&self) -> String {
        fn quote_if_needed(s: &str) -> String {
            if s.contains(' ') || s.contains('\t') {
                format!("\"{s}\"")
            } else {
                s.to_string()
            }
        }

        let mut out = String::new();
        out.push_str(&quote_if_needed(&self.program.display().to_string()));
        for arg in &self.args {
            out.push(' ');
            out.push_str(&quote_if_needed(arg.to_string_lossy().as_ref()));
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct CommandOutputText {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("failed to spawn `{program}`\nargs: {args:?}\ncwd: {cwd:?}\nsource: {source}")]
    Spawn {
        program: String,
        args: Vec<String>,
        cwd: Option<String>,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "`{program}` exited without a status code\nargs: {args:?}\ncwd: {cwd:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    NoStatusCode {
        program: String,
        args: Vec<String>,
        cwd: Option<String>,
        stdout: String,
        stderr: String,
    },
    #[error(
        "`{program}` exited with status {status}\nargs: {args:?}\ncwd: {cwd:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    NonZeroExit {
        program: String,
        args: Vec<String>,
        cwd: Option<String>,
        status: i32,
        stdout: String,
        stderr: String,
    },
}

impl CommandError {
    pub fn program(&self) -> &str {
        match self {
            CommandError::Spawn { program, .. } => program,
            CommandError::NoStatusCode { program, .. } => program,
            CommandError::NonZeroExit { program, .. } => program,
        }
    }
}

pub fn run_command_output_text(spec: &CommandSpec) -> Result<CommandOutputText, CommandError> {
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args);
    if let Some(cwd) = &spec.cwd {
        cmd.current_dir(cwd);
    }

    let output: Output = cmd.output().map_err(|e| CommandError::Spawn {
        program: spec.program.display().to_string(),
        args: spec
            .args
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect(),
        cwd: spec.cwd.as_ref().map(|p| p.display().to_string()),
        source: e,
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let status = match output.status.code() {
        Some(v) => v,
        None => {
            return Err(CommandError::NoStatusCode {
                program: spec.program.display().to_string(),
                args: spec
                    .args
                    .iter()
                    .map(|a| a.to_string_lossy().to_string())
                    .collect(),
                cwd: spec.cwd.as_ref().map(|p| p.display().to_string()),
                stdout,
                stderr,
            });
        }
    };

    Ok(CommandOutputText {
        status,
        stdout,
        stderr,
    })
}

pub fn run_command_expect_success(spec: &CommandSpec) -> Result<CommandOutputText, CommandError> {
    let out = run_command_output_text(spec)?;
    if out.status == 0 {
        Ok(out)
    } else {
        Err(CommandError::NonZeroExit {
            program: spec.program.display().to_string(),
            args: spec
                .args
                .iter()
                .map(|a| a.to_string_lossy().to_string())
                .collect(),
            cwd: spec.cwd.as_ref().map(|p| p.display().to_string()),
            status: out.status,
            stdout: out.stdout,
            stderr: out.stderr,
        })
    }
}

pub fn ensure_parent_dir(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
