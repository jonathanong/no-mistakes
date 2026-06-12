export type RouteName = "users" | "billing" | "settings";

export const ROUTE_META = {
  users: { slug: "users" },
  billing: { slug: "billing" },
} as const;
