# ejaan-rs

> /e.ja.an/<br />
> &nbsp;&nbsp;An Indonesian word meaning "spelling".

A simple Rust + Node.js library for checking spelling using Native system libraries.

## API Used
- Windows: [ISpellChecker](https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nn-spellcheck-ispellchecker)
- macOS: [NSSpellChecker](https://developer.apple.com/documentation/appkit/nsspellchecker?language=objc)
- Linux: Unavailable

## Requirements
- **Windows**: Windows 10 or later
- **macOS**: macOS 10.15 (Catalina) or later
- **Linux**: Not supported
- **Node.js**: v18.17.0+
- **Rust**: v1.85.0+ (for building from source)

## Installation

```bash
npm install @noaione/ejaan-rs
# Or
yarn add @noaione/ejaan-rs
```

## Usages

```javascript
import { SpellChecker } from '@noaione/ejaan-rs';

const spellChecker = new SpellChecker();

// This will throw an error if the language is not installed on your system
// You can also leave it empty, this will auto detect the language based on the system locale
spellChecker.setLanguage('en');

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

## License

This project is dual-licensed under the Apache-2.0 and MIT licenses at your convenience.
