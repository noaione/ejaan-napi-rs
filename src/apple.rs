//! Apple-specific implementation of the spell checker.

use objc2::rc::{Retained, autoreleasepool};
use objc2_app_kit::NSSpellChecker;
use objc2_foundation::{NSRange, NSString};

use crate::{
    SpellCheckerImpl,
    utils::{EjaanError, TokenWithSuggestions, tokenize_sentence},
};

pub struct AppleSpellChecker {
    shared: Retained<NSSpellChecker>,
}

impl AppleSpellChecker {
    /// Creates a shared instance of the Apple spell checker.
    pub fn new() -> Self {
        unsafe {
            let shared = NSSpellChecker::sharedSpellChecker();
            // By default, we guess the language automatically.
            shared.setAutomaticallyIdentifiesLanguages(true);
            Self { shared }
        }
    }

    fn suggest(&self, word: &str) -> Vec<String> {
        unsafe {
            let ns_word = NSString::from_str(word);
            let range = NSRange::new(0, ns_word.len());
            let suggestions = self
                .shared
                .guessesForWordRange_inString_language_inSpellDocumentWithTag(
                    range, &ns_word, None, 0,
                );
            if let Some(suggestions) = suggestions {
                // Convert NSArray to Vec<String>
                let counter = suggestions.count();
                let mut result = Vec::with_capacity(counter);
                for i in 0..counter {
                    let suggestion = suggestions.objectAtIndex(i);
                    autoreleasepool(|pool| {
                        let suggestion_str = suggestion.to_str(pool);
                        result.push(suggestion_str.to_string());
                    })
                }
                result
            } else {
                Vec::new() // No suggestions available
            }
        }
    }
}

impl SpellCheckerImpl for AppleSpellChecker {
    fn add_word(&self, word: &str) -> EjaanError<()> {
        unsafe {
            // &str -> NSString
            let ns_word = NSString::from_str(word);
            self.shared.learnWord(&ns_word);
            Ok(())
        }
    }

    fn remove_word(&self, word: &str) -> EjaanError<()> {
        unsafe {
            // &str -> NSString
            let ns_word = NSString::from_str(word);
            if self.shared.hasLearnedWord(&ns_word) {
                self.shared.unlearnWord(&ns_word);
            }
            Ok(())
        }
    }

    fn set_language(&mut self, language: &str) -> EjaanError<bool> {
        unsafe {
            // &str -> NSString
            let ns_language = NSString::from_str(language);
            Ok(self.shared.setLanguage(&ns_language))
        }
    }

    fn get_available_languages(&self) -> EjaanError<Vec<String>> {
        unsafe {
            let languages = self.shared.availableLanguages();
            // Iterate over the NSArray and convert to Vec<String>
            let counter = languages.count();
            let mut result = Vec::with_capacity(counter);
            for i in 0..counter {
                let lang = languages.objectAtIndex(i);
                autoreleasepool(|pool| {
                    let lang_str = lang.to_str(pool);
                    result.push(lang_str.to_string());
                })
            }

            Ok(result)
        }
    }

    fn check_word(&self, word: &str) -> EjaanError<bool> {
        unsafe {
            let ns_word = NSString::from_str(word);
            let ranges = self.shared.checkSpellingOfString_startingAt(&ns_word, 0);
            // If the range is empty, the word is spelled correctly
            Ok(ranges.is_empty())
        }
    }

    fn check_sentences(&self, sentence: &str) -> EjaanError<Vec<TokenWithSuggestions>> {
        let tokenized = tokenize_sentence(sentence);
        Ok(tokenized
            .iter()
            .filter_map(|token| {
                if self.check_word(token.word()).expect("Failed to check word") {
                    None // Word is spelled correctly, no need to collect
                } else {
                    // Word is misspelled, collect the range
                    Some(TokenWithSuggestions::new(
                        token.clone(),
                        self.suggest(token.word()),
                    ))
                }
            })
            .collect())
    }

    fn get_language(&self) -> EjaanError<Option<String>> {
        unsafe {
            let language = self.shared.language();
            if language.is_empty() {
                Ok(None) // No language set
            } else {
                Ok(Some(language.to_string()))
            }
        }
    }
}
