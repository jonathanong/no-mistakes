import "recovered-dependency";
// This malformed escape produces a recoverable diagnostic and retains imports.
export const diagnostic = '\u{}';
