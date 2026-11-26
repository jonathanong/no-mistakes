declare const reactModule: string;
const R = require(reactModule);

function A(x: R.ReactNode) {
  return x ?? "fallback";
}
