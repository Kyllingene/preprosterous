#![feature(trait_alias)]

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::io;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;
use std::process::exit;
use std::vec::IntoIter;

const VALID_IDENT_CHARS: [char; 63] = [
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
    'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3',
    '4', '5', '6', '7', '8', '9',
];

type Context = HashMap<String, String>;
type MacroResult = Result<CharVec, MacroError>;

#[derive(Debug)]
enum MacroError {
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

/// A vector of Characters, the input/output of most of these functions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CharVec(Vec<Character>);

impl CharVec {
    /// Convert the CharVec into production-ready output.
    ///
    /// This reverts all ::Dollars and ::Percents into their chars.
    pub fn output(&self) -> String {
        self.iter()
            .map(|ch| match ch {
                Character::Dollar => '$',
                Character::Percent => '%',
                Character::Char(ch) => *ch,
            })
            .collect()
    }
}

impl Deref for CharVec {
    type Target = Vec<Character>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CharVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<Character> for CharVec {
    fn from_iter<T: IntoIterator<Item = Character>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<String> for CharVec {
    fn from(s: String) -> Self {
        substitute(&s)
    }
}

impl From<&String> for CharVec {
    fn from(s: &String) -> Self {
        substitute(s)
    }
}

impl IntoIterator for CharVec {
    type Item = Character;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// An invocation/variable marker or a plain character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Character {
    /// The marker for a line invocation.
    Dollar,

    /// The marker for a variable substitution.
    Percent,

    /// A plain or escaped character.
    Char(char),
}

impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Character::Dollar => write!(f, "$")?,
            Character::Percent => write!(f, "%")?,

            Character::Char('$') => write!(f, "\\$")?,
            Character::Char('%') => write!(f, "\\%")?,

            Character::Char('\n') => writeln!(f)?,

            Character::Char(ch) => write!(f, "{ch}")?,
        }
        Ok(())
    }
}

/// Strip the comments from a string.
///
/// A comment is a line starting with "$$".
fn decomment(s: &str) -> String {
    s.lines()
        .filter(|l| !l.starts_with("$$"))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Perform substitution on a string.
///
/// Unescaped `%` will be substituted for Character::Percent,
/// as will `$` at the start of a line.
fn substitute(s: &str) -> CharVec {
    let s: Vec<char> = s.chars().collect();
    let mut new = Vec::new();

    let mut newline = true;
    let mut i = 0;
    while i < s.len() {
        let ch: char = s[i];

        let push = match ch {
            '\\' => {
                if i != s.len() {
                    i += 1;

                    let ch = s[i];
                    match ch {
                        '$' => Character::Char('$'),
                        '%' => Character::Char('%'),
                        _ => {
                            new.push(Character::Char('\\'));
                            Character::Char(ch)
                        }
                    }
                } else {
                    Character::Char('\\')
                }
            }

            '$' => {
                if newline {
                    Character::Dollar
                } else {
                    Character::Char('$')
                }
            }
            '%' => Character::Percent,

            ch => Character::Char(ch),
        };

        new.push(push);

        if ch == '\n' {
            newline = true;
        } else {
            newline = false;
        }

        i += 1;
    }

    CharVec(new)
}

impl Display for CharVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ch in self.iter() {
            write!(f, "{ch}")?;
        }

        Ok(())
    }
}

// TODO: generalize macro invocations
// TODO: prevent circular includes
/// Read a file into memory and process it fully.
///
/// If the file doesn't start with `$PREP enable`, returns an unmodified buffer.
fn process_file(path: impl AsRef<Path>, mut vars: &mut Context) -> MacroResult {
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

            let mut include = include_macro(args, &mut vars)?;
            include = substitute_variables(include, &vars);

            new.push(Character::Char('\n'));
            new.extend(&mut include.iter());
        } else if let Some(args) =
            line.strip_prefix(CharVec::from("$PREP concat ".to_string()).as_slice())
        {
            let args = args
                .split(|ch| *ch == Character::Char(' '))
                .map(|arg| CharVec(arg.to_vec()))
                .collect();

            assert!(concat_macro(args, &mut vars)?.is_empty());
        } else if let Some(args) =
            line.strip_prefix(CharVec::from("$PREP stringify ".to_string()).as_slice())
        {
            let args = args
                .split(|ch| *ch == Character::Char(' '))
                .map(|arg| CharVec(arg.to_vec()))
                .collect();

            assert!(stringify_macro(args, &mut vars)?.is_empty());
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

            let mut define = define_macro(vec![name, args], &mut vars)?;
            define = substitute_variables(define, &mut vars);

            new.push(Character::Char('\n'));
            new.extend(&mut define.iter());
        } else if let Some(cmd) = line.strip_prefix(CharVec::from("$PREP ".to_string()).as_slice())
        {
            return Err(MacroError::UnknownCommand(
                CharVec(cmd.to_vec()).to_string(),
            ));
        } else {
            let line = substitute_variables(CharVec(line.to_vec()), &vars);

            new.push(Character::Char('\n'));
            new.extend(line.iter());
        }
    }

    Ok(CharVec(new.into_iter().skip(1).collect()))
}

/// Find and perform variable substitutions.
fn substitute_variables(s: CharVec, vars: &Context) -> CharVec {
    // TODO: should there be more rigorous syntax enforcement (e.g. not allowing unclosed delimiters)?
    vars.iter().fold(s, |s, (k, v)| {
        s.to_string().replace(&format!("%{k}%"), v).into()
    })
}

/// Execute a `concat` macro.
fn concat_macro(args: Vec<CharVec>, vars: &mut Context) -> MacroResult {
    if args.len() < 2 {
        return Err(MacroError::ExpectedNArgs(2, args.len()));
    }

    let mut args = args.into_iter();

    let name = args.next().unwrap();
    let args: CharVec = args
        .map(|mut cv| {
            cv.push(Character::Char(' '));
            cv.into_iter()
        })
        .flatten()
        .collect();

    let args = substitute_variables(args, &vars);

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
        Character::Char(ch) => VALID_IDENT_CHARS.contains(ch),
        _ => false,
    }) {
        return Err(MacroError::InvalidArg(name.to_string()));
    }

    let name = name.to_string();

    vars.insert(name, rest.output());

    Ok(CharVec(Vec::new()))
}

/// Execute an `include` macro.
fn include_macro(args: Vec<CharVec>, vars: &mut Context) -> MacroResult {
    if args.len() != 1 {
        return Err(MacroError::ExpectedNArgs(1, args.len()));
    }

    let data = process_file(args[0].to_string(), vars)?;

    Ok(data)
}

/// Execute a `stringify` macro.
fn stringify_macro(args: Vec<CharVec>, vars: &mut Context) -> MacroResult {
    if args.len() != 2 {
        return Err(MacroError::ExpectedNArgs(2, args.len()));
    }

    let name = args[0].output();
    let data: CharVec = process_file(args[1].to_string(), vars)?;

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
fn define_macro(args: Vec<CharVec>, vars: &mut Context) -> MacroResult {
    if args.len() != 2 {
        return Err(MacroError::ExpectedNArgs(2, args.len()));
    }

    let mut args = args.into_iter();
    let name = args.next().unwrap();
    let args = args.next().unwrap();

    if !name.iter().all(|ch| match ch {
        Character::Char(ch) => VALID_IDENT_CHARS.contains(ch),
        _ => false,
    }) {
        return Err(MacroError::InvalidArg(name.to_string()));
    }

    let name = name.to_string();

    vars.insert(name, substitute_variables(args, &vars).output());

    Ok(CharVec(Vec::new()))
}

fn main() {
    let mut vars = HashMap::new();

    let mut args = std::env::args();
    if let Some(file) = args.nth(1) {
        let data = match process_file(file, &mut vars) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error: {e}");
                exit(1);
            }
        };
        
        println!("{}", data.output());
    } else {
        eprintln!("No file supplied");
        exit(1);
    }
}
