with ({ undefined: "en" }) {
  new Date().toLocaleString(undefined);
}
function formatEval() {
  eval("var undefined = 'en'");
  return new Date().toLocaleString(undefined);
}
