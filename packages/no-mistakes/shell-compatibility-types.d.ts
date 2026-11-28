import type { Unanalyzable } from "./traversal-types";

export interface BashEmptyArrayNounsetOptions {
  /** Repository-relative globs for scripts targeting Bash before 4.4. Defaults to no files. */
  files?: string[];
  /** How to report unmodeled control-flow, dynamic unset, or loop-widened array shapes. */
  unanalyzable?: Unanalyzable;
}

export interface ZshReadonlyParameterOptions {
  /** Repository-relative globs for additional zsh sources without a zsh shebang or extension. */
  files?: string[];
}
