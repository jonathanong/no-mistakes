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

var varReact = require("react");
export function varNamespace(child: varReact.ReactNode) {
  return child ?? "fallback";
}

var varModule = require(moduleName);
export function varDynamic(child: varModule.ReactNode) {
  return child ?? "fallback";
}

declare const list: { head: { next: unknown } };

let node = list.head;
node = node.next;

var linked = mutual?.x;
var mutual = linked?.y;
