//! Utility for building makefiles
//!
//! Why use this over simple makefile templates?
//!
//! - composable templating: multiple different sources can contribute to a makefile with minimal
//! care needed to prevent interference.
//!
//! - checked creation: assert that there's no typos in rule names or unreachable rules
//!
//! - less awkward than templating: especially for templates that are small enough for direct
//! strings, doing proper escaping can be a chore
//!
//! - allows for global settings (TODO)
//!
//! - limits complexity: makefiles can get cursed. Using this API regulates the features that can
//! be (ab)used
//!
//! - it's kinda cool and I want to

type Str = Box<str>;

pub struct Makefile {

}

pub struct Rule {
    phoney: bool,
    commands: Vec<Command>,
}

pub struct Command {
    args: Vec<CommandArg>,
}

pub enum CommandArg {
    Literal(Str),
    Variable(Str),
}

pub enum Prereq {
    /// Single literal prereq
    Literal(Str),

    /// potentially automatic variable
    Variable(Str),

    /// `%` matching pattern
    Replace(Str)
}

// impl FromStr for Prereq {
//     type Err;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         todo!()
//     }
// }
