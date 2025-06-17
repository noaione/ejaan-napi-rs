# ejaan-rs

> /e.ja.an/<br />
> &nbsp;&nbsp;An Indonesian word meaning "spelling".

A simple Rust + Node.js library for checking spelling using Native system libraries.

## API Used
- Windows: [ISpellChecker2](https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nn-spellcheck-ispellchecker2)
- macOS: [NSSpellChecker](https://developer.apple.com/documentation/appkit/nsspellchecker?language=objc)
- Linux: Not supported

## Requirements
- **Windows**: Windows 10 or later (32-bit or 64-bit)
- **macOS**: macOS 10.15 (Catalina) or later (Intel or Apple Silicon)
- **Node.js**: v18.17.0+
- **Rust**: v1.85.0+ (for building from source)

## Installation

```bash
npm install @noaione/ejaan-rs
# Or
yarn add @noaione/ejaan-rs
```

## Usages

Simple usage example:
```javascript
import { SpellChecker } from '@noaione/ejaan-rs';

// Initialize the spell checker, automatically guess the language based on the system locale
const spellChecker = new SpellChecker();

const results = spellChecker.checkAndSuggest("I'm trying to chekc my speling");
console.log(results);
// [
//  {
//     start: 14,
//     end: 18,
//     word: 'chekc',
//     suggestions: [ 'check', 'Cheka' ]
//   }
// ]
```

Adding custom words:
```javascript
spellChecker.addWords(['Cheka', 'speling']);
// Or:
spellChecker.addWord('Cheka');
```

Removing custom words:
```javascript
spellChecker.removeWords(['Cheka', 'speling']);
// Or:
spellChecker.removeWord('Cheka');
```

Getting supported languages:
```javascript
const languages = spellChecker.availableLanguages();
console.log(languages);
// Depending on the system platform and installed languages
// Output: ['en-US', 'id-ID', 'fr-FR', ...]
```

Getting and setting the current language:
```javascript
const currentLanguage = spellChecker.getLanguage();
console.log(currentLanguage); // e.g., 'en-US'
const result = spellChecker.setLanguage('id-ID'); // Set to Indonesian, make sure it exist first!
console.log(result); // true if successful, false if the language is not supported
```

### Exceptions

All function calls can throw an error, in Rust side the function has been wrapped with `Result<T, E>` type,
where `E` is a custom error type that can be converted to a JavaScript error.

## License

This project is dual-licensed under the [Apache-2.0](https://github.com/noaione/ejaan-napi-rs/blob/master/LICENSE-APACHE) and [MIT](https://github.com/noaione/ejaan-napi-rs/blob/master/LICENSE-MIT) licenses at your convenience.

See the [docs/licenses.md](https://github.com/noaione/ejaan-napi-rs/blob/master/docs/licenses.md) for third-party licenses used in this project.

