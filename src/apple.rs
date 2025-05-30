//! Apple-specific implementation of the spell checker.

use objc2::rc::{Retained, autoreleasepool};
use objc2_app_kit::NSSpellChecker;
use objc2_core_foundation::{
    CFRange, CFString, CFStringTokenizer, CFStringTokenizerTokenType, kCFStringTokenizerUnitWord,
};
use objc2_foundation::{NSRange, NSString};

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
        let tokenized = tokenize_sentence(sentence)?;
        Ok(tokenized
            .iter()
            .filter_map(|token| {
                // Guarantee: This will rarely fails
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

fn tokenize_sentence(sentence: &str) -> EjaanError<Vec<Token>> {
    unsafe {
        let mut tokens = Vec::new();
        let cf_string = CFString::from_str(sentence);
        let tokenizer = CFStringTokenizer::new(
            None,
            Some(&cf_string),
            CFRange::new(0, cf_string.length()),
            kCFStringTokenizerUnitWord,
            None,
        )
        .ok_or(crate::utils::Error::new("Failed to create tokenizer"))?;
        // UTF-8 * 2 => UTF-16 compat
        let const_len = cf_string.length() as usize * 2;

        loop {
            let next_token = tokenizer.advance_to_next_token();

            if next_token == CFStringTokenizerTokenType::None {
                break; // No more tokens
            };

            let range_tokens = tokenizer.current_token_range();
            // Buffer to hold sub-token characters
            let mut buffers = vec![0u16; const_len];
            cf_string.characters(range_tokens, buffers.as_mut_ptr());

            // Trim zero NULL characters then convert to a string slice
            let sub_token_str = String::from_utf16_lossy(&buffers)
                .trim_end_matches('\0')
                .to_string();

            let st_index = range_tokens.location as usize;
            let end_index = st_index + range_tokens.length as usize;
            tokens.push(Token::new(st_index, end_index, sub_token_str));
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_sentence_cj() {
        let sentence = "彼がその本を読んだ時、彼はその内容に深く感動した。";
        let tokens = tokenize_sentence(sentence).expect("Failed to tokenize sentence");
        assert!(
            !tokens.is_empty(),
            "Tokenization should not return empty vector"
        );
        assert_eq!(
            tokens.len(),
            17,
            "Expected 17 tokens for the given sentence"
        );
    }

    #[test]
    fn test_tokenize_standard() {
        let sentence = "This is a test sentence.";
        let tokens = tokenize_sentence(sentence).expect("Failed to tokenize sentence");
        assert!(
            !tokens.is_empty(),
            "Tokenization should not return empty vector"
        );
        assert_eq!(tokens.len(), 5, "Expected 5 tokens for the given sentence");
        assert_eq!(tokens[0].word(), "This", "First token should be 'This'");
        assert_eq!(
            tokens[4].word(),
            "sentence",
            "Last token should be 'sentence'"
        );
    }

    #[test]
    fn test_mixed_sentences() {
        let sentence = "彼は本を読んで、I love programming.";
        let tokens = tokenize_sentence(sentence).expect("Failed to tokenize sentence");
        assert!(
            !tokens.is_empty(),
            "Tokenization should not return empty vector"
        );
        assert_eq!(tokens.len(), 9, "Expected 8 tokens for the mixed sentence");
        assert_eq!(tokens[0].word(), "彼", "First token should be '彼'");
        assert_eq!(
            tokens[8].word(),
            "programming",
            "Last token should be 'programming'"
        );
    }

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

        assert!(
            !tokens.is_empty(),
            "Spell checking should not return empty vector"
        );
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
