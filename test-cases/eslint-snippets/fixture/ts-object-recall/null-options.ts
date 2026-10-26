function render(options: { label?: string | null }, key: string) {
  const alias = options;
  const { [`label`]: text } = options;
  alias["label"] ?? "title";
  options[`label`] || "title";
  text ?? "title";
  options[key] ?? "title"; // A computed identifier does not prove a nullable member.
}
// Neither the default value nor an imported type proves the parameter's nullable members.
function defaulted({ label } = options) { return label ?? "title"; }
import type { Options } from "./types";
function imported(options: Options) { return options.label ?? "title"; }
