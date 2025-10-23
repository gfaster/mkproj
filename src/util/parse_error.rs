use core::fmt::{self, Formatter};
use std::error::Error as StdErr;
use std::str::FromStr;
use std::{fmt::Display, ops::Range};

/// general parse error that can give nice messages
pub struct ParseError(Box<ParseErrorInner>);

impl ParseError {
    #[cold]
    fn new(
        input: impl ToString,
        msg: (impl ParseErrorMessage + Send + 'static),
        pos: Option<Range<usize>>,
    ) -> Self {
        ParseError(Box::new(ParseErrorInner {
            input: input.to_string(),
            msg: Box::new(msg),
            pos,
        }))
    }
}

struct ParseErrorInner {
    input: String,
    msg: Box<dyn ParseErrorMessage + Send>,
    pos: Option<Range<usize>>,
}

enum MsgKind {
    Help,
    Error,
}

enum Span {
    Range(Range<usize>),
    Pair(Range<usize>, Range<usize>)
}

struct Msg {
    span: Option<Span>,
    kind: MsgKind,
    msg: String,
}

pub struct MsgBuilder<'i, 'f> {
    input: &'i str,
    span: Option<Span>,
    formatter: &'f mut Formatter<'f>
}


pub struct ErrorBuilder<'i> {
    input: &'i str,
    msg: Vec<Msg>,
}

#[derive(Debug, Default)]
pub struct EnabledFunctionality {
    pub substr: bool,
    pub whole_input: bool,
    pub find_pos: bool,
}

pub trait ParseErrorMessage {
    #![allow(unused_variables)]

    fn enabled_functionality(&self) -> EnabledFunctionality;

    fn find_pos(&self, input: &str) -> Option<Range<usize>> { unimplemented!() }

    /// only called if `self.enabled_formatters().whole_input` is `true`
    ///
    /// Note that this only should be implemented if error message itself needs the whole input.
    fn from_whole_input(&self, f: &mut Formatter, input: &str) -> fmt::Result {
        unimplemented!()
    }

    /// only called if `self.enabled_formatters().substr` is `true`
    ///
    /// Note that this only should be implemented if error message itself needs the position.
    fn from_substr(&self, f: &mut Formatter, input: &str, pos: Range<usize>) -> fmt::Result {
        unimplemented!()
    }

    /// fallback/self-contained error
    fn fallback(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("Unknown parse error")
    }
}

impl ParseErrorMessage for &'static str {
    fn enabled_functionality(&self) -> EnabledFunctionality {
        EnabledFunctionality::default()
    }

    fn fallback(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(*self)
    }
}

fn msg<'a, E>(e: &'a E, input: &'a str, pos: Option<Range<usize>>) -> impl fmt::Display + use<'a, E>
where
    E: ParseErrorMessage + ?Sized,
{
    super::fmt_fn(move |f| {
        let EnabledFunctionality {
            whole_input,
            substr,
            find_pos,
        } = e.enabled_functionality();
        let pos = find_pos.then(|| e.find_pos(input)).flatten().or(pos.clone());
        if let Some(pos) = pos.clone() {
            if substr {
                return e.from_substr(f, input, pos);
            }
        }
        if whole_input {
            return e.from_whole_input(f, input);
        }

        e.fallback(f)
    })
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        msg(&*self.0.msg, &self.0.input, self.0.pos.clone()).fmt(f)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        msg(&*self.0.msg, &self.0.input, self.0.pos.clone()).fmt(f)
    }
}

impl StdErr for ParseError {}

pub struct ParseMessageStdError<E>(E);

impl<E: StdErr> ParseErrorMessage for ParseMessageStdError<E> {
    fn enabled_functionality(&self) -> EnabledFunctionality {
        EnabledFunctionality::default()
    }

    fn fallback(&self, f: &mut Formatter) -> fmt::Result {
        <E as fmt::Display>::fmt(&self.0, f)
    }
}

/// attempt an operation on a substring, mapping the error on failure.
fn try_substr<F, T, E>(input: &str, substr: Range<usize>, f: F) -> Result<T, ParseError>
where
    F: FnOnce(&str) -> Result<T, E>,
    E: StdErr + Send + 'static,
{
    match f(&input[substr.clone()]) {
        Ok(x) => Ok(x),
        Err(e) => Err(ParseError(Box::new(ParseErrorInner {
            input: input.into(),
            msg: Box::new(ParseMessageStdError(e)),
            pos: Some(substr),
        }))),
    }
}

pub trait StdParseExt {
    fn parse_pe<T: FromStrExt>(&self, substr: Range<usize>) -> Result<T, ParseError>
    where
        <T as FromStr>::Err: StdErr + Send + 'static;
}

impl StdParseExt for str {
    fn parse_pe<T: FromStrExt>(&self, substr: Range<usize>) -> Result<T, ParseError>
    where
        <T as FromStr>::Err: StdErr + Send + 'static,
    {
        T::from_str_pe(self, substr)
    }
}

pub trait FromStrExt: FromStr
where
    <Self as FromStr>::Err: StdErr + Send + 'static,
{
    fn from_str_pe(input: &str, substr: Range<usize>) -> Result<Self, ParseError> {
        try_substr(input, substr, Self::from_str)
    }
}

impl<T> FromStrExt for T
where
    T: FromStr,
    T::Err: StdErr + Send + 'static,
{
}

pub trait ParseErrorContext<T, E> {
    fn parse_context(self, input: impl ToString, pos: Range<usize>) -> Result<T, ParseError>
    where
        E: StdErr + Send + 'static;

    fn parse_context_with(
        self,
        input: impl ToString,
        pos: Option<Range<usize>>,
        msg: impl ParseErrorMessage + Send + 'static,
    ) -> Result<T, ParseError>;
}

impl<T, E> ParseErrorContext<T, E> for Result<T, E> {
    #[inline]
    fn parse_context(self, input: impl ToString, pos: Range<usize>) -> Result<T, ParseError>
    where
        E: StdErr + Send + 'static,
    {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(ParseError::new(input, ParseMessageStdError(e), Some(pos))),
        }
    }

    #[inline]
    fn parse_context_with(
        self,
        input: impl ToString,
        pos: Option<Range<usize>>,
        msg: impl ParseErrorMessage + Send + 'static,
    ) -> Result<T, ParseError> {
        debug_assert!(pos.is_some() || !msg.enabled_functionality().substr);
        match self {
            Ok(x) => Ok(x),
            Err(_e) => Err(ParseError::new(input, msg, pos)),
        }
    }
}

/// ParseError for an undeclared varaible
pub struct UnknownVariable;

impl ParseErrorMessage for UnknownVariable {
    fn enabled_functionality(&self) -> EnabledFunctionality {
        EnabledFunctionality {
            substr: true,
            ..EnabledFunctionality::default()
        }
    }

    fn from_substr(&self, f: &mut Formatter, input: &str, pos: Range<usize>) -> fmt::Result {
        write!(f, "unknown variable: `{}'", &input[pos])
    }

    fn fallback(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("unknown variable")
    }
}


pub struct UnbalancedGroup<G: Grouping>(pub G);

impl<G: Grouping> ParseErrorMessage for UnbalancedGroup<G> {
    fn enabled_functionality(&self) -> EnabledFunctionality {
        todo!()
    }
}

pub enum GroupingChar {
    /// contains the corresponding closing char
    Open(char),
    Close,
}

pub trait Grouping {
    /// find the next grouping char, either opening or closing, and return the position along with
    /// the kind
    fn next_grouping_char(&self, s: &str) -> Option<(usize, GroupingChar)>;
}

impl Grouping for (&'static [char], &'static [char]) {
    fn next_grouping_char(&self, s: &str) -> Option<(usize, GroupingChar)> {
        let (open, close) = *self;
        debug_assert_eq!(open.len(), close.len());

        for (i, c) in s.char_indices() {
            if let Some(pos) = open.iter().position(|&open| open == c) {
                return Some((i, GroupingChar::Open(close[pos])))
            }
            if close.contains(&c) {
                return Some((i, GroupingChar::Close))
            }
        }

        None
    }
}


pub struct NormalGrouping;

impl Grouping for NormalGrouping {
    fn next_grouping_char(&self, s: &str) -> Option<(usize, GroupingChar)> {
        (&['(', '{', '['][..], &[')', '}', ']'][..]).next_grouping_char(s)
    }
}
