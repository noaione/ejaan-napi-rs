//! Apple-specific implementation of the spell checker.

use std::ptr::NonNull;

use objc2::rc::{Retained, autoreleasepool};
use objc2_app_kit::NSSpellChecker;
use objc2_foundation::{NSRange, NSString, NSTextCheckingType};

use crate::{
    SpellCheckerImpl,
    utils::{EjaanError, Token, TokenWithSuggestions},
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

    fn suggest<S: AsRef<str>>(&self, word: S) -> Vec<String> {
        unsafe {
            let ns_word = NSString::from_str(word.as_ref());
            let range = NSRange::new(0, ns_word.len());
            let language = self.shared.language();
            let suggestions = self
                .shared
                .guessesForWordRange_inString_language_inSpellDocumentWithTag(
                    range,
                    &ns_word,
                    Some(&language),
                    0,
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
        let ns_string = NSString::from_str(sentence);

        unsafe {
            let mut numbers: isize = 0;
            let mispellings = self
                .shared
                .checkString_range_types_options_inSpellDocumentWithTag_orthography_wordCount(
                    &ns_string,
                    NSRange::new(0, ns_string.length()),
                    NSTextCheckingType::Spelling.0,
                    None,
                    0,
                    None,
                    &mut numbers,
                );

            let mut misspelling =
                Vec::with_capacity(numbers.try_into().unwrap_or(ns_string.length()));
            for i in 0..mispellings.count() {
                let result = mispellings.objectAtIndex(i);
                let ranges = result.range();
                if ranges.is_empty() {
                    // In case the range is empty, skip this result
                    continue;
                }

                let buffer_size = ranges.length.saturating_mul(2);
                let mut buffers = vec![0u16; buffer_size];
                ns_string.getCharacters_range(
                    NonNull::new(buffers.as_mut_ptr()).ok_or(crate::utils::Error::new(format!(
                        "Failed to initialize buffer for misspelled word at range: {:#?}",
                        ranges
                    )))?,
                    ranges,
                );
                let text_data = String::from_utf16_lossy(&buffers)
                    .trim_end_matches('\0')
                    .to_string();

                let st_index = ranges.location as usize;
                let end_index = (st_index + ranges.length as usize).saturating_sub(1);
                let suggestions = self.suggest(&text_data);
                misspelling.push(TokenWithSuggestions::new(
                    Token::new(st_index, end_index, text_data),
                    suggestions,
                ));
            }

            // Trim the size of capacity until the actual length
            misspelling.shrink_to_fit();
            Ok(misspelling)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_spellcheck() {
        let spell_checker = AppleSpellChecker::new();
        let word = "test";
        let is_correct = spell_checker
            .check_word(word)
            .expect("Failed to check word");
        assert!(
            is_correct,
            "The word '{}' should be spelled correctly",
            word
        );
    }

    #[test]
    fn test_simple_sentences() {
        let spell_checker = AppleSpellChecker::new();
        let sentence = "This is a test sentence.";
        let tokens = spell_checker
            .check_sentences(sentence)
            .expect("Failed to check sentences");

        println!("Tokens: {:?}", tokens);

        assert_eq!(
            tokens.len(),
            0,
            "Expected no misspelled words in the sentence"
        );
    }

    #[test]
    fn test_simple_sentences_with_typos() {
        let spell_checker = AppleSpellChecker::new();
        let sentence = "This is a tset sentence.";
        let tokens = spell_checker
            .check_sentences(sentence)
            .expect("Failed to check sentences");

        assert!(
            !tokens.is_empty(),
            "Spell checking should return misspelled words"
        );
        assert_eq!(
            tokens.len(),
            1,
            "Expected one misspelled word in the sentence"
        );
        assert_eq!(
            tokens[0].token().word(),
            "tset",
            "Expected the misspelled word to be 'tset'"
        );
        assert!(
            !tokens[0].suggestions().is_empty(),
            "Expected suggestions for the misspelled word"
        );
    }
}
