// Does not parse; a `__tests__` file never counts as construction, so it
// cannot hide one and does not stop the rule.
const unterminated = {;
