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

#[cfg(target_os = "macos")]
pub fn tokenize_sentence(sentence: &str) -> Vec<Token> {
    // Super quick and naive sentences splitter
    let mut tokens = Vec::new();
    // Also get the ranges of each words, zero-indexed
    let mut current_word = String::new();
    for (i, chars) in sentence.chars().enumerate() {
        if chars.is_whitespace() {
            if !current_word.is_empty() {
                tokens.push(Token::new(
                    i - current_word.len(),
                    i - 1,
                    current_word.clone(),
                ));
                current_word.clear();
            }
            continue;
        }
        current_word.push(chars);
    }
    if !current_word.is_empty() {
        tokens.push(Token::new(
            sentence.len() - current_word.len(),
            sentence.len() - 1,
            current_word,
        ));
    }
    tokens
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

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(target_os = "macos")]
    fn test_tokenize_sentence() {
        use super::Token;

        let sentence = "Hello, world! This is a test.";
        let tokens = super::tokenize_sentence(sentence);
        println!("{:?}", tokens);
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], Token::new(0, 5, "Hello,".to_string()));
        assert_eq!(tokens[1], Token::new(7, 12, "world!".to_string()));
        assert_eq!(tokens[2], Token::new(14, 17, "This".to_string()));
        assert_eq!(tokens[3], Token::new(19, 20, "is".to_string()));
        assert_eq!(tokens[4], Token::new(22, 22, "a".to_string()));
        assert_eq!(tokens[5], Token::new(24, 28, "test.".to_string()));
    }
}
