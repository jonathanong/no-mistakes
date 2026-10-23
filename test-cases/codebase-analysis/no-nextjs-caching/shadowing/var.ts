const mode = 'force-cache';
function blockVar() {
  function earlier() { fetch('/before-var', { cache: mode }); }
  const alias = mode;
  if (condition) { var mode = 'no-store'; }
  fetch('/after-var', { cache: alias });
  fetch('/after-block', { cache: mode });
}
function loopVar() {
  function earlier() { fetch('/before-loop-var', { cache: mode }); }
  for (var mode of items) { fetch('/loop-var', { cache: mode }); }
  fetch('/after-loop-var', { cache: mode });
}
function nestedVar() {
  function inner() { if (condition) { var mode = 'no-store'; } }
  const arrow = () => { var mode = 'no-store'; };
  class Inner {
    method() { var mode = 'no-store'; }
    static {
      fetch('/static-before-var', { cache: mode });
      if (condition) { var mode = 'no-store'; }
    }
    static other = fetch('/after-static-block', { cache: mode }); // finding
  }
  fetch('/outer', { cache: mode }); // finding
}
const arrow = () => {
  fetch('/arrow-before-var', { cache: mode });
  { var mode = 'no-store'; }
};
function destructuredVar() {
  fetch('/destructured-before-var', { cache: mode });
  { var { mode } = source; }
}
