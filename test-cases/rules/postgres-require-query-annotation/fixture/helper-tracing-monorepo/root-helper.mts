// The root alias must not answer package-local helper imports.
export function statement() { return "/* root_query */ SELECT 999"; }
