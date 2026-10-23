const mode = 'force-cache';
function functionDeclaration() {
  function earlier() { fetch('/before-function', { cache: mode }); }
  const alias = mode;
  function mode() {}
  fetch('/function', { cache: alias });
}
function classDeclaration() {
  function earlier() { fetch('/before-class', { cache: mode }); }
  const alias = mode;
  class mode {}
  fetch('/class', { cache: alias });
}
function blockDeclaration() {
  { function earlier() { fetch('/block-function', { cache: mode }); } function mode() {} }
  { function earlier() { fetch('/block-class', { cache: mode }); } class mode {} }
  fetch('/after-block', { cache: mode }); // finding
}
// Named expressions bind only inside themselves.
const namedFunction = function mode() { fetch('/expression-function', { cache: mode }); };
const namedClass = class mode { method() { fetch('/expression-class', { cache: mode }); } };
fetch('/after-expressions', { cache: mode }); // finding
