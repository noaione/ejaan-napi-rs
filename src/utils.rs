use std::ops::RangeInclusive;

pub type EjaanError<T> = Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    start: usize,
    end: usize,
    word: String,
}

impl Token {
    pub(crate) fn new(start: usize, end: usize, word: String) -> Self {
        Token { start, end, word }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn range(&self) -> RangeInclusive<usize> {
        self.start..=self.end
    }
}

#[derive(Debug, Clone)]
pub struct TokenWithSuggestions {
    token: Token,
    suggestions: Vec<String>,
}

impl TokenWithSuggestions {
    pub(crate) fn new(token: Token, suggestions: Vec<String>) -> Self {
        TokenWithSuggestions { token, suggestions }
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn suggestions(&self) -> &[String] {
        &self.suggestions
    }
}

impl std::ops::Deref for TokenWithSuggestions {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

/// Error type for the spell checker
#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error {
            message: message.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<Error> for napi::Error {
    fn from(err: Error) -> Self {
        napi::Error::from_reason(err.message)
    }
}

#[cfg(target_os = "windows")]
impl From<windows::core::Error> for Error {
    fn from(err: windows::core::Error) -> Self {
        Error::new(&format!(
            "Windows error: {} (code: {})",
            err.message(),
            err.code().0
        ))
    }
}


