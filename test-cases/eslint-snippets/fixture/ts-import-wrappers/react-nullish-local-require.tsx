function require(name: string): { ReactNode: string } {
  return { ReactNode: name };
}
const Local = require("react");
export function localRequire(props: { child: Local.ReactNode }) {
  return props.child ?? "fallback";
}
