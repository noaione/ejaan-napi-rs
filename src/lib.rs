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
///
/// As a sidenote, all API returned can throw an error, especially on Windows.
///
/// @class SpellChecker
#[napi]
pub struct SpellChecker {
    inner: Box<dyn SpellCheckerImpl>,
}

/// A suggestion for a misspelled word.
///
/// @typedef {Object} Suggestion
/// @property {number} start The start index of the string
/// @property {number} end The end index of the string
/// @property {string} word The misspelled word
/// @property {string[]} suggestions The list of suggested words
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
    /// The main Spell checker class.
    ///
    /// This automatically determine the appropriate spell checker implementation based on the platform.
    ///
    /// As a sidenote, all API returned can throw an error, especially on Windows.
    ///
    /// @returns {void}
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
    ///
    /// @returns {string | null}
    #[napi]
    pub fn language(&self) -> napi::Result<Option<String>> {
        Ok(self.inner.get_language()?)
    }

    /// Set the language for the spell checker.
    ///
    /// @param {string} language The preferred spell checker language.
    /// @returns {void}
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
    /// @returns {string[]} A list of available languages.
    #[napi]
    pub fn available_languages(&self) -> napi::Result<Vec<String>> {
        Ok(self.inner.get_available_languages()?)
    }

    /// Check if a word is spelled correctly.
    ///
    /// @param {string} word The word to check
    /// @returns {boolean} Is the word spelled correctly or not.
    #[napi]
    pub fn check_word(&self, word: String) -> napi::Result<bool> {
        Ok(self.inner.check_word(&word)?)
    }

    /// Check if a word is spelled correctly.
    ///
    /// This will also return a list of suggestions if the word is misspelled.
    ///
    /// @param {string} sentences The sentences to check
    /// @returns {Suggestion[]} The list of suggested spellings.
    #[napi]
    pub fn check_and_suggest(&self, sentences: String) -> napi::Result<Vec<JsSuggestion>> {
        let tokens = self.inner.check_sentences(&sentences)?;

        Ok(tokens.into_iter().map(JsSuggestion::from).collect())
    }

    /// Add a single word to the spell checker.
    ///
    /// ## Implementation Note
    /// On Windows, this will add the word to the IGNORE list instead of the dictionary.
    /// This is done to avoid adding the word permanently to the dictionary,
    ///
    /// @param {string} word The word to add
    /// @returns {void}
    #[napi]
    pub fn add_word(&self, word: String) -> napi::Result<()> {
        self.inner.add_word(&word)?;
        Ok(())
    }

    /// Add words to the spell checker.
    ///
    /// ## Implementation Note
    /// On Windows, this will add the word to the IGNORE list instead of the dictionary.
    /// This is done to avoid adding the word permanently to the dictionary,
    ///
    /// @param {string[]} words The words to add
    /// @returns {void}
    #[napi]
    pub fn add_words(&self, words: Vec<String>) -> napi::Result<()> {
        self.inner.add_words(words)?;
        Ok(())
    }

    /// Remove a single word from the spell checker.
    ///
    /// ## Implementation Note
    /// On Windows, this will be ignored.
    ///
    /// @param {string} word The word to remove
    /// @returns {void}
    #[napi]
    pub fn remove_word(&self, word: String) -> napi::Result<()> {
        self.inner.remove_word(&word)?;
        Ok(())
    }

    /// Remove words from the spell checker.
    ///
    /// ## Implementation Note
    /// On Windows, this will be ignored.
    ///
    /// @param {string[]} words The words to remove
    /// @returns {void}
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
