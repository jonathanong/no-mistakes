const mode = 'force-cache';
// Parameter initializers do not see declarations owned by the function body.
function bodyVar(input = fetch('/parameter', { cache: mode })) { // finding
  if (condition) { var mode = 'no-store'; }
  fetch('/body', { cache: mode });
}
const arrow = (input = fetch('/arrow-parameter', { cache: mode })) => { // finding
  { var mode = 'no-store'; }
  fetch('/arrow-body', { cache: mode });
};
function parameterBinding(mode, input = fetch('/other-parameter', { cache: mode })) {
  fetch('/bound-body', { cache: mode });
}
