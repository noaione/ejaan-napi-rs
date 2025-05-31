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
  t.is(result.word, 'snetences'); // since windows and mac has some differs
  t.is(result.start, 18);
  t.is(result.end, 26);
  
  t.true(result.suggestions.length > 0, 'Suggestions should not be empty');
});

test('can handle utf-8 characters just fine', (t) => {
  const spellchecker = new SpellChecker();

  try {
    // Make sure it has zero result and it didn't fails
    const results = spellchecker.checkAndSuggest('“As I said, this should work properly!”');

    t.is(results.length, 0, 'No misspelled words should be found');
  } catch (error) {
    t.fail(`SpellChecker failed with error: ${error.message}`);
  }

  // Now with error
  try {
    const results = spellchecker.checkAndSuggest('“As I said, this should work properly!” with a mistake snetences.');

    t.is(results.length, 1, 'One misspelled word should be found');
    const result = results[0];
    t.is(result.word, 'snetences');
    t.is(result.start, 55);
    t.is(result.end, 63);
    t.true(result.suggestions.length > 0, 'Suggestions should not be empty');
  } catch (error) {
    t.fail(`SpellChecker failed with error: ${error.message}`);
  }
})
