export const TABLES = ["users", "posts"] as const;
export type Table = (typeof TABLES)[number];
export const TABLE_NAMES = { user: "users", post: "posts" } as const;
export const PARTITIONED = [
  { table: "users", column: "created_at" },
  { table: "posts", column: "published_at" },
] as const;
