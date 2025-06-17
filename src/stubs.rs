//! Fake spell checker implementation for unsupported platforms.

use crate::SpellCheckerImpl;

pub struct StubSpellChecker;

impl StubSpellChecker {
    /// Creates a new instance of the stub spell checker.
    #[allow(dead_code)]
    pub fn new() -> Self {
        StubSpellChecker
    }
}

impl SpellCheckerImpl for StubSpellChecker {
    fn check_word(&self, _word: &str) -> crate::utils::EjaanError<bool> {
        Err(crate::utils::Error::unimplemented())
    }

    fn check_sentences(
        &self,
        _sentence: &str,
    ) -> crate::utils::EjaanError<Vec<crate::utils::TokenWithSuggestions>> {
        Err(crate::utils::Error::unimplemented())
    }

    fn add_word(&self, _word: &str) -> crate::utils::EjaanError<()> {
        Err(crate::utils::Error::unimplemented())
    }

    fn add_words(&self, _words: Vec<String>) -> crate::utils::EjaanError<()> {
        Err(crate::utils::Error::unimplemented())
    }

    fn remove_word(&self, _word: &str) -> crate::utils::EjaanError<()> {
        Err(crate::utils::Error::unimplemented())
    }

    fn remove_words(&self, _words: Vec<String>) -> crate::utils::EjaanError<()> {
        Err(crate::utils::Error::unimplemented())
    }

    fn get_available_languages(&self) -> crate::utils::EjaanError<Vec<String>> {
        Err(crate::utils::Error::unimplemented())
    }

    fn get_language(&self) -> crate::utils::EjaanError<Option<String>> {
        Err(crate::utils::Error::unimplemented())
    }

    fn set_language(&mut self, _language: &str) -> crate::utils::EjaanError<bool> {
        Err(crate::utils::Error::unimplemented())
    }
}
