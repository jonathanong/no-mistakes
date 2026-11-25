export const extra = ["**/*.slow.test.ts"] as const;
export const gitMetadata = ["**/.git/**"] as const;

export const changedPatterns = ["**/.git/**"];
changedPatterns.pop();
