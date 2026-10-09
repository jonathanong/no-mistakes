// The disposable argument helper forces the callback's mapped frame to move.
function discard() {
  return "/* captured mapped value */ SELECT 1";
}
function factory(parameter) {
  return () => parameter;
}
const callback = condition ? factory(discard()) : "";
