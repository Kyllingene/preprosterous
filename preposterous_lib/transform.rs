use crate::{
    character::{CharVec, Character},
    types::Context,
};

/// All the valid variable ident characters `/[_a-zA-Z0-9]/`.
pub const VALID_IDENT_CHARS: [char; 63] = [
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
    'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3',
    '4', '5', '6', '7', '8', '9',
];

/// Strip the comments from a string.
///
/// A comment is a line starting with "$$".
pub fn decomment(s: &str) -> String {
    s.lines()
        .filter(|l| !l.starts_with("$$"))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Perform substitution on a string.
///
/// Unescaped `%` will be substituted for Character::Percent,
/// as will `$` at the start of a line.
pub fn substitute(s: &str) -> CharVec {
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

/// Find and perform variable substitutions.
pub fn substitute_variables(s: CharVec, vars: &Context) -> CharVec {
    // TODO: should there be more rigorous syntax enforcement (e.g. not allowing unclosed delimiters)?
    vars.iter().fold(s, |s, (k, v)| {
        s.to_string().replace(&format!("%{k}%"), v).into()
    })
}
