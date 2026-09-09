// These helper calls deliberately share a source line but are different AST calls.
export const value = () => helper(); helper();
