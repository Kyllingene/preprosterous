//! The module containing the macro definitions.
//!
//! *WARNING:* EVERYTHING in this module gets `use`d in lib.rs.

use crate::{
    character::{CharVec, Character},
    transform,
    types::{Context, MacroError, MacroResult},
};

/// Execute a `concat` macro.
pub fn concat_macro(args: Vec<CharVec>, vars: &mut Context) -> MacroResult {
    if args.len() < 2 {
        return Err(MacroError::ExpectedNArgs(2, args.len()));
    }

    let mut args = args.into_iter();

    let name = args.next().unwrap();
    let args: CharVec = args
        .flat_map(|mut cv| {
            cv.push(Character::Char(' '));
            cv.into_iter()
        })
        .collect();

    let args = transform::substitute_variables(args, vars);

    let (open, close) = if let Some(open) = vars.get(&"%OPENING_STRINGIFY_DELIMITER".to_string()) {
        if let Some(close) = vars.get(&"%CLOSING_STRINGIFY_DELIMITER".to_string()) {
            (open.clone(), close.clone())
        } else {
            ("\"".to_string(), "\"".to_string())
        }
    } else {
        ("\"".to_string(), "\"".to_string())
    };

    let mut rest: CharVec = open.into();

    let mut i = 0;
    while i < args.len() {
        let ch = args[i];
        match ch {
            Character::Dollar => rest.push(Character::Char('$')),
            Character::Percent => rest.push(Character::Char('%')),

            Character::Char('\\') => {
                i += 1;
                if i < args.len() {
                    rest.push(ch);
                    rest.push(args[i]);
                }
            }

            Character::Char('"') => {}

            _ => rest.push(ch),
        }

        i += 1;
    }

    rest.extend::<CharVec>(close.into());

    if !name.iter().all(|ch| match ch {
        Character::Char(ch) => transform::VALID_IDENT_CHARS.contains(ch),
        _ => false,
    }) {
        return Err(MacroError::InvalidArg(name.to_string()));
    }

    let name = name.to_string();

    vars.insert(name, rest.output());

    Ok(CharVec(Vec::new()))
}

/// Execute an `include` macro.
pub fn include_macro(args: Vec<CharVec>, vars: &mut Context) -> MacroResult {
    if args.len() != 1 {
        return Err(MacroError::ExpectedNArgs(1, args.len()));
    }

    let data = crate::process_file(args[0].to_string(), vars)?;

    Ok(data)
}

/// Execute a `stringify` macro.
pub fn stringify_macro(args: Vec<CharVec>, vars: &mut Context) -> MacroResult {
    if args.len() != 2 {
        return Err(MacroError::ExpectedNArgs(2, args.len()));
    }

    let name = args[0].output();
    let data: CharVec = crate::process_file(args[1].to_string(), vars)?;

    let (open, close) = if let Some(open) = vars.get(&"%OPENING_STRINGIFY_DELIMITER".to_string()) {
        if let Some(close) = vars.get(&"%CLOSING_STRINGIFY_DELIMITER".to_string()) {
            (open.clone(), close.clone())
        } else {
            ("\"".to_string(), "\"".to_string())
        }
    } else {
        ("\"".to_string(), "\"".to_string())
    };

    let mut val = Vec::with_capacity(
        data.len()
            + open.len()
            + close.len()
            + data
                .iter()
                .filter(|ch| matches!(ch, Character::Char('\n') | Character::Char('"')))
                .count(),
    );
    val.extend(&mut open.chars().map(Character::Char));

    for &ch in data.iter() {
        match ch {
            Character::Char('\n') | Character::Char('"') => {
                val.push(Character::Char('\\'));
                val.push(ch);
            }

            _ => val.push(ch),
        }
    }

    val.extend(&mut close.chars().map(Character::Char));

    vars.insert(name, CharVec(val).output());

    Ok(CharVec(Vec::new()))
}

/// Execute a `define` macro.
pub fn define_macro(args: Vec<CharVec>, vars: &mut Context) -> MacroResult {
    if args.len() != 2 {
        return Err(MacroError::ExpectedNArgs(2, args.len()));
    }

    let mut args = args.into_iter();
    let name = args.next().unwrap();
    let args = args.next().unwrap();

    if !name.iter().all(|ch| match ch {
        Character::Char(ch) => transform::VALID_IDENT_CHARS.contains(ch),
        _ => false,
    }) {
        return Err(MacroError::InvalidArg(name.to_string()));
    }

    let name = name.to_string();

    vars.insert(name, transform::substitute_variables(args, vars).output());

    Ok(CharVec(Vec::new()))
}
