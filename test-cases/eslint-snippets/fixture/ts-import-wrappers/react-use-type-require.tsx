import type { require } from "./types";
const React = require("react");
export function view(value: Promise<string>) {
  return React.use(Promise.resolve(value));
}
