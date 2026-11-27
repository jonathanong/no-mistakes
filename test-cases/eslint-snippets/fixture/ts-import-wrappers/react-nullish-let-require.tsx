declare const moduleName: string;

let R = require("react");
export function letNamespace(child: R.ReactNode) {
  return child ?? "fallback";
}

let Dynamic = require(moduleName);
export function letDynamic(child: Dynamic.ReactNode) {
  return child ?? "fallback";
}

let Written = require("react");
Written = null;
export function letWritten(child: Written.ReactNode) {
  return child ?? "fallback";
}
