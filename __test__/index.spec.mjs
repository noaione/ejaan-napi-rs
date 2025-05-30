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
  console.log(results);

  // Make sure it has single result
  t.is(results.length, 1);

  // Concents
  const result = results[0];
  t.is(result.word, 'snetences'); // since windows and mac has some differs
  t.is(result.start, 18);
  t.is(result.end, 26);
  
  t.true(result.suggestions.length > 0, 'Suggestions should not be empty');
});
