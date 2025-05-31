# Changelog

The following file contains all the changes made in `@noaione/ejaan-rs` package.

## Unreleased
### Improvements
- Use [`checkString:range:types:options:inSpellDocumentWithTag:orthography:wordCount:`](https://developer.apple.com/documentation/appkit/nsspellchecker/check(_:range:types:options:inspelldocumentwithtag:orthography:wordcount:)?language=objc) for spell-checking sentences
- Fix Windows substring extraction issues when it get non-ASCII characters

## [0.1.1] 2025-05-30
### Improvements
- Replace naive tokenizer in macOS with [`CFStringTokenizer`](https://developer.apple.com/documentation/corefoundation/cfstringtokenizercreate(_:_:_:_:_:)?language=objc) for better accurary.

### Docs
- Better explain some stuff in the typings

## [0.1.0] 2025-05-30
This is the first release of `@noaione/ejaan-rs` package.
- Initial release with basic functionality
- Supports native Windows and macOS spell checking
