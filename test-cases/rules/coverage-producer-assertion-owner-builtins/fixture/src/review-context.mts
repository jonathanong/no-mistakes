export function numericComparator(rows: number[]) {
  rows.sort((left, right) => { left.toFixed(); right.toExponential(); return 0; });
}
export function stringComparator(words: string[]) {
  words.toSorted(function (left, right) { left.localeCompare(right); right.trim(); return 0; });
}
export function stringChain(words: string[]) {
  return words.map(text => text.trim().toLowerCase()).join(',');
}
export function lastIndexContext(values: string[]) {
  values.findLastIndex(word => word.trim());
}
export function writtenString(words: string[], factory: () => unknown) {
  words.forEach(text => { text = factory(); text.trim(); });
}
export function writtenPattern(words: string[]) {
  words.forEach(text => { ({ text } = { text: 'changed' }); text.trim(); });
  words.forEach(text => { [text] = ['changed']; text.trim(); });
}
export function writtenExpression(words: string[]) {
  words.forEach(text => (text = 'changed', text.trim()));
}
export function writtenUpdate(updateNumbers: number[]) {
  updateNumbers.forEach(value => { value++; value.toFixed(); });
}
export function callbackShapeCases(words: string[]) {
  words.map(function (word: unknown) { word.trim(); });
  words.map(function (word) { { const word: unknown = 'shadow'; word.trim(); } });
  words.map(word => String(word).trim());
}
