//! Windows-specific implementation of the spell checker.

use windows::{
    Win32::{
        Foundation::S_OK,
        Globalization::{
            CORRECTIVE_ACTION_DELETE, CORRECTIVE_ACTION_GET_SUGGESTIONS, CORRECTIVE_ACTION_NONE,
            CORRECTIVE_ACTION_REPLACE, GetUserDefaultLocaleName, ISpellChecker2,
            ISpellCheckerFactory, LOCALE_NAME_SYSTEM_DEFAULT, SpellCheckerFactory,
        },
        System::Com::{
            CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
        },
    },
    core::{HSTRING, Interface, PCWSTR, PWSTR},
};

use crate::{
    SpellCheckerImpl,
    utils::{EjaanError, Token, TokenWithSuggestions},
};

pub struct WindowsSpellChecker {
    inner: ISpellCheckerFactory,
    checker: ISpellChecker2,
    locale: String,
}

impl WindowsSpellChecker {
    /// Create a new instance of the Windows spell checker.
    pub fn new() -> EjaanError<Self> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };

        let inner: ISpellCheckerFactory =
            unsafe { CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_ALL)? };

        let mut locale_name = [0u16; 256];
        unsafe { GetUserDefaultLocaleName(&mut locale_name) };
        let all_null = locale_name.iter().all(|&c| c == 0);

        let locale_name = if all_null {
            LOCALE_NAME_SYSTEM_DEFAULT
        } else {
            PCWSTR::from_raw(locale_name.as_ptr())
        };

        let checker = Self::make_spell_checker(&inner, locale_name)?;

        let locale = if all_null {
            "en-US".to_string() // Default to English if no locale is set
        } else {
            let locale_wide = unsafe {
                // Convert the locale name from wide string to UTF-8
                locale_name.as_wide()
            };
            String::from_utf16_lossy(locale_wide)
                .trim_end_matches('\0')
                .to_string()
        };

        Ok(Self {
            inner,
            checker,
            locale,
        })
    }

    fn make_spell_checker(
        factory: &ISpellCheckerFactory,
        locale: PCWSTR,
    ) -> EjaanError<ISpellChecker2> {
        let checker = unsafe { factory.CreateSpellChecker(locale)? };

        Ok(checker.cast::<ISpellChecker2>()?)
    }

    fn common_spellcheck(&self, word: &str) -> EjaanError<Vec<TokenWithSuggestions>> {
        let mut tokens = Vec::new();

        let wide_word = HSTRING::from(word);

        let errors = unsafe { self.checker.Check(&wide_word)? };
        loop {
            let mut error = None;
            if unsafe { errors.Next(&mut error) } != S_OK {
                break;
            }

            let err = error.ok_or_else(|| {
                crate::utils::Error::new("Failed to retrieve next error from spell checker")
            })?;

            let start_index = unsafe { err.StartIndex()? };
            let length = unsafe { err.Length()? };

            let range = start_index as usize..(start_index + length) as usize;
            let substring = wide_word
                .get(range.clone())
                .ok_or(crate::utils::Error::new(format!(
                    "Failed to get substring for range: {:?}",
                    range
                )))?;
            let action = unsafe { err.CorrectiveAction()? };

            let substring = String::from_utf16_lossy(substring)
                .trim_end_matches('\0')
                .to_string();

            let token = Token::new(
                start_index as usize,
                (start_index + length) as usize - 1,
                substring.to_string(),
            );
            match action {
                CORRECTIVE_ACTION_DELETE | CORRECTIVE_ACTION_NONE => {
                    // If the action is to delete, we don't add a token
                    continue;
                }
                CORRECTIVE_ACTION_GET_SUGGESTIONS => {
                    let suggestions = unsafe { self.checker.Suggest(&HSTRING::from(substring))? };

                    let mut tokenized_suggest = Vec::new();
                    loop {
                        let mut suggestion = [PWSTR::null()];
                        unsafe {
                            _ = suggestions.Next(&mut suggestion, None);
                        }

                        if suggestion[0].is_null() {
                            unsafe { CoTaskMemFree(Some(suggestion[0].as_ptr() as *mut _)) };
                            break;
                        }

                        let suggest_str = unsafe {
                            suggestion[0].to_string().map_err(|e| {
                                crate::utils::Error::new(format!(
                                    "Failed to convert suggestion PWSTR to string: {}",
                                    e
                                ))
                            })?
                        };
                        tokenized_suggest.push(suggest_str);

                        unsafe { CoTaskMemFree(Some(suggestion[0].as_ptr() as *mut _)) };
                    }

                    tokens.push(TokenWithSuggestions::new(token, tokenized_suggest));
                }
                CORRECTIVE_ACTION_REPLACE => {
                    let repl = unsafe { err.Replacement()? };
                    // PWSTR -> string
                    let repl_str = unsafe {
                        repl.to_string().map_err(|e| {
                            crate::utils::Error::new(format!(
                                "Failed to convert replacement PWSTR to string: {}",
                                e
                            ))
                        })?
                    };

                    unsafe { CoTaskMemFree(Some(repl.as_ptr() as *mut _)) };

                    tokens.push(TokenWithSuggestions::new(token, vec![repl_str]))
                }
                _ => {}
            }
        }

        Ok(tokens)
    }
}

impl SpellCheckerImpl for WindowsSpellChecker {
    fn get_available_languages(&self) -> EjaanError<Vec<String>> {
        let mut merged = Vec::new();
        let results = unsafe { self.inner.SupportedLanguages()? };

        loop {
            let mut suggestion = [PWSTR::null()];
            _ = unsafe { results.Next(&mut suggestion, None) };
            if suggestion[0].is_null() {
                break;
            }

            let lang_str = unsafe { suggestion[0].to_string() }.map_err(|e| {
                crate::utils::Error::new(format!(
                    "Failed to convert language PWSTR to string: {}",
                    e
                ))
            })?;

            merged.push(lang_str);

            unsafe { CoTaskMemFree(Some(suggestion[0].as_ptr() as *mut _)) };
        }

        Ok(merged)
    }

    fn check_word(&self, word: &str) -> EjaanError<bool> {
        let tokens = self.common_spellcheck(word)?;
        Ok(tokens.is_empty())
    }

    fn check_sentences(&self, sentence: &str) -> EjaanError<Vec<TokenWithSuggestions>> {
        self.common_spellcheck(sentence)
    }

    fn add_word(&self, word: &str) -> EjaanError<()> {
        let wide_word = word.encode_utf16().collect::<Vec<u16>>();
        let ptr = PCWSTR::from_raw(wide_word.as_ptr());
        // > Use Ignore instead of Add.
        // Since according to MSFT themselves, Ignore will only happens
        // only on the current checker instances itself rather than updating
        // globally.
        unsafe { self.checker.Ignore(ptr) }?;

        Ok(())
    }

    fn remove_word(&self, word: &str) -> EjaanError<()> {
        let wide_word = word.encode_utf16().collect::<Vec<u16>>();
        let ptr = PCWSTR::from_raw(wide_word.as_ptr());

        unsafe { self.checker.Remove(ptr)? };

        Ok(())
    }

    fn get_language(&self) -> EjaanError<Option<String>> {
        Ok(Some(self.locale.clone()))
    }

    fn set_language(&mut self, language: &str) -> EjaanError<bool> {
        let locale = PCWSTR::from_raw(language.encode_utf16().collect::<Vec<u16>>().as_ptr());

        let ret = unsafe { self.inner.IsSupported(locale)? };
        if ret.as_bool() {
            // Change the spell checker language
            self.checker = Self::make_spell_checker(&self.inner, locale)?;
            self.locale = language.to_string();

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_spellcheck() {
        let spell_checker = WindowsSpellChecker::new().unwrap();
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
        let spell_checker = WindowsSpellChecker::new().unwrap();
        let sentence = "This is a test sentence.";
        let tokens = spell_checker
            .check_sentences(sentence)
            .expect("Failed to check sentences");

        assert_eq!(
            tokens.len(),
            0,
            "Expected no misspelled words in the sentence"
        );
    }

    #[test]
    fn test_simple_sentences_with_typos() {
        let spell_checker = WindowsSpellChecker::new().unwrap();
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

    #[test]
    fn test_utf_8_characters() {
        let spell_checker = WindowsSpellChecker::new().unwrap();
        let word = "“Test...”";

        let tokens = spell_checker
            .check_sentences(word)
            .expect("Failed to check sentences");

        assert_eq!(
            tokens.len(),
            0,
            "Expected no misspelled words in the UTF-8 characters sentence"
        );

        let word = "“Tset...”";
        let tokens = spell_checker
            .check_sentences(word)
            .expect("Failed to check sentences");

        assert!(
            !tokens.is_empty(),
            "Spell checking should return misspelled words for UTF-8 characters"
        );
    }
}
