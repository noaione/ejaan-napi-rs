use napi_derive::*;

use crate::utils::{EjaanError, TokenWithSuggestions};

#[cfg(target_os = "macos")]
mod apple;
mod utils;
#[cfg(target_os = "windows")]
mod winrt;

/// The main trait for spell checking functionality.
pub trait SpellCheckerImpl {
    /// Check if a word is spelled correctly.
    ///
    /// # Arguments
    /// * `word` - The word to check.
    ///
    /// # Returns
    /// A boolean indicating whether the word is spelled correctly.
    fn check_word(&self, word: &str) -> EjaanError<bool>;
    /// Check if a sentence is spelled correctly.
    ///
    /// # Arguments
    /// * `sentence` - The sentence to check.
    ///
    /// # Returns
    /// A list of index positions where the words are misspelled.
    fn check_sentences(&self, sentence: &str) -> EjaanError<Vec<TokenWithSuggestions>>;

    /// Add a word to the spell checker.
    fn add_word(&self, word: &str) -> EjaanError<()>;
    /// Remove a word from the spell checker.
    ///
    /// This will silently fail if the word is not found.
    fn remove_word(&self, word: &str) -> EjaanError<()>;

    /// Batch add words to the spell checker.
    ///
    /// # Arguments
    /// * `words` - A list of words to add.
    fn add_words(&self, words: Vec<String>) -> EjaanError<()> {
        for word in words {
            self.add_word(&word)?;
        }
        Ok(())
    }

    /// Batch remove words from the spell checker.
    ///
    /// # Arguments
    /// * `words` - A list of words to remove.
    fn remove_words(&self, words: Vec<String>) -> EjaanError<()> {
        for word in words {
            self.remove_word(&word)?;
        }
        Ok(())
    }

    /// Get a list of available languages for the spell checker.
    fn get_available_languages(&self) -> EjaanError<Vec<String>>;

    /// Get the current language of the spell checker.
    fn get_language(&self) -> EjaanError<Option<String>>;
    /// Set the language for the spell checker.
    fn set_language(&mut self, language: &str) -> EjaanError<bool>;
}

/// The main Spell checker class.
///
/// This automatically determine the appropriate spell checker implementation based on the platform.
#[napi]
pub struct SpellChecker {
    inner: Box<dyn SpellCheckerImpl>,
}

/// A suggestion for a misspelled word.
#[napi(object, js_name = "Suggestion")]
pub struct JsSuggestion {
    /// The start index of the misspelled word in the original text.
    pub start: u32,
    /// The end index of the misspelled word in the original text.
    pub end: u32,
    /// The misspelled word.
    pub word: String,
    /// A list of suggested corrections for the misspelled word.
    pub suggestions: Vec<String>,
}

#[napi]
impl SpellChecker {
    /// Create a new instance of the spell checker.
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        #[cfg(target_os = "macos")]
        let inner = apple::AppleSpellChecker::new();
        #[cfg(target_os = "windows")]
        let inner = winrt::WindowsSpellChecker::new().map_err(|e| {
            napi::Error::from_reason(format!(
                "Failed to create Windows spell checker: {}",
                e.message()
            ))
        })?;

        Ok(Self {
            inner: Box::new(inner),
        })
    }

    /// Get the current language of the spell checker.
    #[napi]
    pub fn language(&self) -> napi::Result<Option<String>> {
        Ok(self.inner.get_language()?)
    }

    /// Set the language for the spell checker.
    #[napi]
    pub fn set_language(&mut self, language: String) -> napi::Result<()> {
        if !self.inner.set_language(&language)? {
            return Err(napi::Error::from_reason(format!(
                "Failed to set language: {}",
                language
            )));
        }
        Ok(())
    }

    /// Get the list of available languages for the spell checker.
    ///
    /// # Returns
    /// A list of available languages.
    #[napi]
    pub fn available_languages(&self) -> napi::Result<Vec<String>> {
        Ok(self.inner.get_available_languages()?)
    }

    /// Check if a word is spelled correctly.
    ///
    /// # Arguments
    /// * `word` - The word to check.
    #[napi]
    pub fn check_word(&self, word: String) -> napi::Result<bool> {
        Ok(self.inner.check_word(&word)?)
    }

    /// Check if a word is spelled correctly.
    ///
    /// This will also return a list of suggestions if the word is misspelled.
    ///
    /// # Arguments
    /// * `sentences` - The sentence to check.
    #[napi]
    pub fn check_and_suggest(&self, sentences: String) -> napi::Result<Vec<JsSuggestion>> {
        let tokens = self.inner.check_sentences(&sentences)?;

        Ok(tokens.into_iter().map(JsSuggestion::from).collect())
    }

    /// Add a single word to the spell checker.
    ///
    /// # Arguments
    /// * `word` - The word to add.
    #[napi]
    pub fn add_word(&self, word: String) -> napi::Result<()> {
        self.inner.add_word(&word)?;
        Ok(())
    }

    /// Add a words to the spell checker.
    ///
    /// # Arguments
    /// * `words` - The words to add.
    #[napi]
    pub fn add_words(&self, words: Vec<String>) -> napi::Result<()> {
        self.inner.add_words(words)?;
        Ok(())
    }

    /// Remove a single word from the spell checker.
    ///
    /// # Arguments
    /// * `word` - The word to remove.
    ///
    /// # Note
    /// On Windows, this will always just silently fails.
    #[napi]
    pub fn remove_word(&self, word: String) -> napi::Result<()> {
        self.inner.remove_word(&word)?;
        Ok(())
    }

    /// Remove words from the spell checker.
    ///
    /// # Arguments
    /// * `words` - The words to remove.
    ///
    /// # Note
    /// On Windows, this will always just silently fails.
    #[napi]
    pub fn remove_words(&self, words: Vec<String>) -> napi::Result<()> {
        self.inner.remove_words(words)?;
        Ok(())
    }
}

impl From<TokenWithSuggestions> for JsSuggestion {
    fn from(token: TokenWithSuggestions) -> Self {
        JsSuggestion {
            start: token.start().try_into().unwrap_or(0),
            end: token.end().try_into().unwrap_or(0),
            word: token.word().to_string(),
            suggestions: token.suggestions().to_vec(),
        }
    }
}
