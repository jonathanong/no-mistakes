// Max resolves this body, but configured grammar rejects nonliteral and extra arguments.
export function staticNamed(_sql: string) { return '/* generic static helper */ SELECT 1'; }
