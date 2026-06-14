export type RouteName = "users" | "billing" | "settings";

export const ROUTE_META = {
  users: { slug: "users" },
  billing: { slug: "billing" },
} as const;

export const FIRST_PARTY_EXEMPTIONS = [
  { name: "@acme/api", reason: "workspace package" },
  { name: "@acme/web", reason: "workspace package" },
  { name: "@acme/docs", reason: "workspace package" },
] as const;

export const FIRST_PARTY_NAMES = [
  // dynamic templates are intentionally skipped by the finite-set extractor
  `@acme/${pkg}`,
  "@acme/api",
  /* documented in docs/dependency-updates.md */
  "@acme/web",
] as const;
