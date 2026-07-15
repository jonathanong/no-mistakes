// Only a bare-specifier re-export: proves bare specifiers are left unresolved and
// contribute no tagged names, even though the target module obviously exists.
export * from "vitest";
