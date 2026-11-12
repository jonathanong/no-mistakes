export const TABLE_NAMES = ["users", "events"] as const;
export type TableName = (typeof TABLE_NAMES)[number];
