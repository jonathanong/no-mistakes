import type { NamedOptions } from "./options-helpers";
export function evaluated(options: NamedOptions) {}
eval("evaluated = options => options.query('SELECT hidden')");
