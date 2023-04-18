use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;
use std::vec::IntoIter;

use crate::transform::substitute;

/// An invocation/variable marker or a plain character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Character {
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

/// A vector of `Character`s, the input/output of most `prep` functions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharVec(pub Vec<Character>);

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

impl Display for CharVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ch in self.iter() {
            write!(f, "{ch}")?;
        }

        Ok(())
    }
}
