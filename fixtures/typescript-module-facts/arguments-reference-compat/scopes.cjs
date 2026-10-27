// Legacy reference metadata crosses implicit arguments environments; evaluation does not.
const arguments = 'outer';
const named = function arguments(seed = arguments) {
  arguments;
  (() => arguments)();
  function nested() { return arguments; }
  { let arguments = 'block'; arguments; function blockNested() { return arguments; } }
  return nested;
};
function outerCapture() { return arguments; }
function parameter(arguments) { return arguments; }
function laterBody(seed = arguments) { var arguments = 'body'; return arguments; }
function nestedDefault(seed = () => arguments) { var arguments = 'body'; return arguments; }
function destructured({[arguments]: value} = {}) { let arguments = 'body'; return value; }
const redeclared = function arguments() { var arguments; arguments = 'write'; return arguments; };
try { throw {}; } catch (arguments) {
  arguments;
  function caught() { return arguments; }
}
const arrow = (arguments) => arguments;
const outerArrow = () => arguments;
class Methods { method(seed = arguments) { return arguments; } }
// Existing and restored references interleave across nested lexical environments.
const arrowOwner = (arguments) => {
  arguments;
  function inner(seed = arguments) { return arguments; }
  return arguments;
};
