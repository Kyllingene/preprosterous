use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::io;

use crate::CharVec;

/// A context for variable evaluation.
pub type Context = HashMap<String, String>;

/// A Result carrying data to insert into the output.
pub type MacroResult = Result<CharVec, MacroError>;

#[derive(Debug)]
pub enum MacroError {
    ExpectedNArgs(usize, usize),
    InvalidArg(String),
    UnknownCommand(String),

    IoError(u32, io::Error),
}

impl Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedNArgs(expected, got) => {
                write!(f, "Expected {expected} arguments, got {got}")
            }
            Self::InvalidArg(arg) => write!(f, "Invalid argument: `{arg}`"),
            Self::UnknownCommand(cmd) => write!(f, "Unknown command: `{cmd}`"),
            Self::IoError(lineno, e) => write!(f, "(at line#{lineno}) {e}"),
        }
    }
}

impl Error for MacroError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::IoError(_, e) = self {
            Some(e)
        } else {
            None
        }
    }
}
