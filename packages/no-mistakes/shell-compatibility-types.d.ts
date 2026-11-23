export interface BashEmptyArrayNounsetOptions {
  /** Repository-relative globs for scripts targeting Bash before 4.4. Defaults to no files. */
  files?: string[];
}

export interface ZshReadonlyParameterOptions {
  /** Repository-relative globs for additional zsh sources without a zsh shebang or extension. */
  files?: string[];
}
