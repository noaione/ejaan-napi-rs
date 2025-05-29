import test from 'ava'

import { SpellChecker } from '../index.js';

test('is valid', (t) => {
  const spellchecker = new SpellChecker();

  const result = spellchecker.checkAndSuggest('This is a valid sentences.');
  t.deepEqual(result, []);
});

test('has invalid', (t) => {
  const spellchecker = new SpellChecker();

  const results = spellchecker.checkAndSuggest('This is a invalid snetences.');

  // Make sure it has single result
  t.is(results.length, 1);

  // Concents
  const result = results[0];
  t.is(result.word.replace('.', ''), 'snetences'); // since windows and mac has some differs
  t.is(result.start, 18);
  t.true(result.end === 26 || result.end === 27);
  
  t.true(result.suggestions.length > 0, 'Suggestions should not be empty');
});
