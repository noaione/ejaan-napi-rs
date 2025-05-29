//! Windows-specific implementation of the spell checker.

use windows::{
    Win32::{
        Foundation::S_OK,
        Globalization::{
            CORRECTIVE_ACTION_DELETE, CORRECTIVE_ACTION_GET_SUGGESTIONS, CORRECTIVE_ACTION_NONE,
            CORRECTIVE_ACTION_REPLACE, GetUserDefaultLocaleName, ISpellChecker,
            ISpellCheckerFactory, LOCALE_NAME_SYSTEM_DEFAULT, SpellCheckerFactory,
        },
        System::Com::{
            CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
        },
    },
    core::{HSTRING, PCWSTR, PWSTR},
};

use crate::{
    SpellCheckerImpl,
    utils::{EjaanError, Token, TokenWithSuggestions},
};

pub struct WindowsSpellChecker {
    inner: ISpellCheckerFactory,
    checker: ISpellChecker,
    locale: String,
}

impl WindowsSpellChecker {
    /// Create a new instance of the Windows spell checker.
    pub fn new() -> EjaanError<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

            let inner: ISpellCheckerFactory =
                CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_ALL)?;

            let mut locale_name = [0u16; 256];
            GetUserDefaultLocaleName(&mut locale_name);
            let all_null = locale_name.iter().all(|&c| c == 0);

            let locale_name = if all_null {
                LOCALE_NAME_SYSTEM_DEFAULT
            } else {
                PCWSTR::from_raw(locale_name.as_ptr())
            };

            let checker = inner.CreateSpellChecker(locale_name)?;

            let locale = if all_null {
                "en-US".to_string() // Default to English if no locale is set
            } else {
                String::from_utf16_lossy(locale_name.as_wide())
                    .trim_end_matches('\0')
                    .to_string()
            };

            Ok(Self {
                inner,
                checker,
                locale,
            })
        }
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
            let substring = word.get(range).ok_or_else(|| {
                crate::utils::Error::new("Failed to get substring for spell check token")
            })?;
            let action = unsafe { err.CorrectiveAction()? };

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
                            break;
                        }

                        let suggest_str = unsafe {
                            suggestion[0].to_string().map_err(|e| {
                                crate::utils::Error::new(&format!(
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
                            crate::utils::Error::new(&format!(
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
        unsafe {
            let mut merged = Vec::new();
            let results = self.inner.SupportedLanguages()?;

            loop {
                let mut suggestion = [PWSTR::null()];
                _ = results.Next(&mut suggestion, None);
                if suggestion[0].is_null() {
                    break;
                }

                let lang_str = suggestion[0].to_string().map_err(|e| {
                    crate::utils::Error::new(&format!(
                        "Failed to convert language PWSTR to string: {}",
                        e
                    ))
                })?;

                merged.push(lang_str);

                CoTaskMemFree(Some(suggestion[0].as_ptr() as *mut _));
            }

            Ok(merged)
        }
    }

    fn check_word(&self, word: &str) -> EjaanError<bool> {
        let tokens = self.common_spellcheck(word)?;
        Ok(tokens.is_empty())
    }

    fn check_sentences(&self, sentence: &str) -> EjaanError<Vec<TokenWithSuggestions>> {
        self.common_spellcheck(sentence)
    }

    fn add_word(&self, word: &str) -> EjaanError<()> {
        unsafe {
            let wide_word = word.encode_utf16().collect::<Vec<u16>>();
            let ptr = PCWSTR::from_raw(wide_word.as_ptr());
            // > Use Ignore instead of Add.
            // Since according to MSFT themselves, Ignore will only happens
            // only on the current checker instances itself rather than updating
            // globally.
            self.checker.Ignore(ptr)?;

            Ok(())
        }
    }

    fn remove_word(&self, _: &str) -> EjaanError<()> {
        Err(crate::utils::Error::new(
            "`remove_word` is not implemented for WindowsSpellChecker",
        ))
    }

    fn get_language(&self) -> EjaanError<Option<String>> {
        Ok(Some(self.locale.clone()))
    }

    fn set_language(&mut self, language: &str) -> EjaanError<bool> {
        unsafe {
            let locale = PCWSTR::from_raw(language.encode_utf16().collect::<Vec<u16>>().as_ptr());

            let ret = self.inner.IsSupported(locale)?;
            if ret.as_bool() {
                // Change the spell checker language
                self.checker = self.inner.CreateSpellChecker(locale)?;
                self.locale = language.to_string();

                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{SpellCheckerImpl, winrt::WindowsSpellChecker};

    #[test]
    fn test_ignored_text() {
        let dict = WindowsSpellChecker::new().unwrap();
        dict.add_word("broekn").unwrap();
        let results = dict.check_sentences("is this broekn for you?").unwrap();
        println!("{:?}", results);
    }
}
