use std::fmt::{Debug, Display, Formatter};

#[macro_export]
macro_rules! log {
    ($($tokens: tt)*) => {
        for line in format!($($tokens)*).lines() {
            println!("cargo::warning={}", line)
        }
    }
}

#[derive(Debug)]
pub struct FatalBuildScriptError {
    pub message: String,
}

impl Display for FatalBuildScriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "lockjaw fatal build script error: {}", self.message)
    }
}

impl std::error::Error for FatalBuildScriptError {}

#[macro_export]
macro_rules! build_script_fatal {
    ($($tokens: tt)*) => {
        return Err(crate::build_log::FatalBuildScriptError{message: format!($($tokens)*)}.into())
    }
}
