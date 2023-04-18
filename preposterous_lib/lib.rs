#![doc = include_str!("README.md")]

use std::fs;
use std::path::Path;

pub mod character;
pub mod macros;
pub mod transform;
pub mod types;

use character::{CharVec, Character};
use macros::*;
use transform::{decomment, substitute, substitute_variables};
use types::{Context, MacroError, MacroResult};

/// Re-exports the types required to run preposterous.
pub mod prelude {
    pub use crate::{
        character::{CharVec, Character},
        process_file,
        types::{Context, MacroError},
    };
}

// TODO: generalize macro invocations
// TODO: prevent circular includes
/// Read a file into memory and process it fully.
///
/// If the file doesn't start with `$PREP enable`, returns an unmodified buffer.
pub fn process_file(path: impl AsRef<Path>, vars: &mut Context) -> MacroResult {
    let data = fs::read_to_string(path).map_err(|e| MacroError::IoError(line!(), e))?;

    let data = if let Some(d) = data.strip_prefix("$PREP enable\n") {
        substitute(&decomment(d))
    } else {
        return Ok(data.chars().map(Character::Char).collect());
    };

    let mut new = Vec::new();
    for line in data.split(|ch| *ch == Character::Char('\n')) {
        if line.starts_with(&CharVec::from("$PREP disable".to_string())) {
            break;
        } else if let Some(args) =
            line.strip_prefix(CharVec::from("$PREP include ".to_string()).as_slice())
        {
            let args = args
                .split(|ch| *ch == Character::Char(' '))
                .map(|arg| CharVec(arg.to_vec()))
                .collect();

            let mut include = include_macro(args, vars)?;
            include = substitute_variables(include, vars);

            new.push(Character::Char('\n'));
            new.extend(&mut include.iter());
        } else if let Some(args) =
            line.strip_prefix(CharVec::from("$PREP concat ".to_string()).as_slice())
        {
            let args = args
                .split(|ch| *ch == Character::Char(' '))
                .map(|arg| CharVec(arg.to_vec()))
                .collect();

            assert!(concat_macro(args, vars)?.is_empty());
        } else if let Some(args) =
            line.strip_prefix(CharVec::from("$PREP stringify ".to_string()).as_slice())
        {
            let args = args
                .split(|ch| *ch == Character::Char(' '))
                .map(|arg| CharVec(arg.to_vec()))
                .collect();

            assert!(stringify_macro(args, vars)?.is_empty());
        } else if let Some(args) =
            line.strip_prefix(CharVec::from("$PREP define ".to_string()).as_slice())
        {
            let (space, _) = args
                .iter()
                .enumerate()
                .find(|(_, ch)| **ch == Character::Char(' '))
                .ok_or(MacroError::ExpectedNArgs(2, 0))?;

            let (name, args) = args.split_at(space);

            let name = CharVec(name.to_vec());
            let args = CharVec(args.iter().skip(1).cloned().collect());

            let mut define = define_macro(vec![name, args], vars)?;
            define = substitute_variables(define, vars);

            new.push(Character::Char('\n'));
            new.extend(&mut define.iter());
        } else if let Some(cmd) = line.strip_prefix(CharVec::from("$PREP ".to_string()).as_slice())
        {
            return Err(MacroError::UnknownCommand(
                CharVec(cmd.to_vec()).to_string(),
            ));
        } else {
            let line = substitute_variables(CharVec(line.to_vec()), vars);

            new.push(Character::Char('\n'));
            new.extend(line.iter());
        }
    }

    Ok(CharVec(new.into_iter().skip(1).collect()))
}
