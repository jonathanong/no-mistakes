import type { BashEmptyArrayNounsetOptions, ZshReadonlyParameterOptions } from "no-mistakes";

export const legacyBash: BashEmptyArrayNounsetOptions = {
  files: ["scripts/legacy/**/*.sh"],
  unanalyzable: "advise",
};
export const zshSources: ZshReadonlyParameterOptions = { files: ["scripts/startup"] };
