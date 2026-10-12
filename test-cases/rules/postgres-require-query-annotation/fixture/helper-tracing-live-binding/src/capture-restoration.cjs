function make(statement) {
  let current = statement;
  return () => current;
}

function discard() {
  return '/* disposable frame */ SELECT 1';
}

const original = make('/* original capture */ SELECT 1');
// The first arm creates a capture after a disposable frame; the other arm
// retains the original capture. The nested choice must preserve both readers.
const first = flag ? make(discard()) : original;
const second = other ? (nested ? first : make('/* nested capture */ SELECT 2')) : original;
