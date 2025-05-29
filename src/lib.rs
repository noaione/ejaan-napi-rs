use napi_derive::*;

use crate::utils::Token;

#[cfg(target_os = "macos")]
mod apple;
mod utils;
#[cfg(target_os = "windows")]
mod windows;

/// The main trait for spell checking functionality.
pub trait SpellCheckerImpl {
    /// Check if a word is spelled correctly.
    /// 
    /// # Arguments
    /// * `word` - The word to check.
    /// 
    /// # Returns
    /// A boolean indicating whether the word is spelled correctly.
    fn check_word(&self, word: &str) -> bool;
    /// Check if a sentence is spelled correctly.
    /// 
    /// # Arguments
    /// * `sentence` - The sentence to check.
    /// 
    /// # Returns
    /// A list of index positions where the words are misspelled.
    fn check_sentence(&self, sentence: &str) -> Vec<Token>;
    /// Suggest corrections for a misspelled word.
    /// 
    /// Can return an empty list if no suggestions are available.
    fn suggest(&self, word: &str) -> Vec<String>;

    /// Add a word to the spell checker.
    fn add_word(&self, word: &str) -> ();
    /// Remove a word from the spell checker.
    /// 
    /// This will silently fail if the word is not found.
    fn remove_word(&self, word: &str) -> ();

    /// Batch add words to the spell checker.
    /// 
    /// # Arguments
    /// * `words` - A list of words to add.
    fn add_words(&self, words: Vec<String>) -> () {
        for word in words {
            self.add_word(&word);
        }
    }

    /// Batch remove words from the spell checker.
    /// 
    /// # Arguments
    /// * `words` - A list of words to remove.
    fn remove_words(&self, words: Vec<String>) -> () {
        for word in words {
            self.remove_word(&word);
        }
    }

    /// Get a list of available languages for the spell checker.
    fn get_available_languages(&self) -> Vec<String>;

    /// Get the current language of the spell checker.
    fn get_language(&self) -> Option<String>;
    /// Set the language for the spell checker.
    fn set_language(&self, language: &str) -> bool;
}

#[napi]
pub struct SpellChecker {
    inner: Box<dyn SpellCheckerImpl>,
}

#[napi(object, js_name = "Suggestion")]
pub struct JsSuggestion {
    pub start: u32,
    pub end: u32,
    pub word: String,
    pub suggestions: Vec<String>,
}

#[napi]
impl SpellChecker {
    #[napi(constructor)]
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        let inner = apple::AppleSpellChecker::new();
        #[cfg(target_os = "windows")]
        let inner = windows::WindowsSpellChecker::new();

        Self {
            inner: Box::new(inner),
        }
    }

    /// Get the current language of the spell checker.
    #[napi]
    pub fn language(&self) -> napi::Result<Option<String>> {
        Ok(self.inner.get_language())
    }

    /// Set the language for the spell checker.
    #[napi]
    pub fn set_language(&self, language: String) -> napi::Result<()> {
        if !self.inner.set_language(&language) {
            return Err(napi::Error::from_reason(format!("Failed to set language: {}", language)));
        }
        Ok(())
    }

    /// Get the list of available languages for the spell checker.
    /// 
    /// # Returns
    /// A list of available languages.
    #[napi]
    pub fn available_languages(&self) -> napi::Result<Vec<String>> {
        Ok(self.inner.get_available_languages())
    }

    /// Check if a word is spelled correctly.
    /// 
    /// # Arguments
    /// * `word` - The word to check.
    #[napi]
    pub fn check_word(&self, word: String) -> napi::Result<bool> {
        Ok(self.inner.check_word(&word))
    }

    /// Suggest corrections for a misspelled word.
    /// 
    /// # Arguments
    /// * `word` - The word to suggest corrections for.
    pub fn suggest(&self, word: String) -> napi::Result<Vec<String>> {
        Ok(self.inner.suggest(&word))
    }

    /// Check if a word is spelled correctly.
    /// 
    /// This will also return a list of suggestions if the word is misspelled.
    /// 
    /// # Arguments
    /// * `sentences` - The sentence to check.
    #[napi]
    pub fn check_and_suggest(&self, sentences: String) -> napi::Result<Vec<JsSuggestion>> {
        let mut suggestions: Vec<JsSuggestion> = Vec::new();
        let tokens = self.inner.check_sentence(&sentences);

        for token in tokens {
            let suggested= self.inner.suggest(token.word());
            suggestions.push(JsSuggestion {
                start: token.start().try_into().map_err(|_| napi::Error::from_reason("Invalid start index"))?,
                end: token.end().try_into().map_err(|_| napi::Error::from_reason("Invalid end index"))?,
                word: token.word().to_string(),
                suggestions: suggested,
            })
        }

        Ok(suggestions)
    }

    /// Add a word to the spell checker.
    /// 
    /// # Arguments
    /// * `words` - The word to add.
    #[napi]
    pub fn add_words(&self, words: Vec<String>) -> napi::Result<()> {
        self.inner.add_words(words);
        Ok(())
    }

    /// Remove a word from the spell checker.
    /// 
    /// # Arguments
    /// * `words` - The word to remove.
    #[napi]
    pub fn remove_words(&self, words: Vec<String>) -> napi::Result<()> {
        self.inner.remove_words(words);
        Ok(())
    }
}
