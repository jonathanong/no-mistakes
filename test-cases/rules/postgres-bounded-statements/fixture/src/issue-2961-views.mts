import { query } from "@example/db";

export function users() {
  return query("SELECT id FROM view_users_public");
}

export function usersCapped() {
  return query("SELECT id FROM view_users_public LIMIT 1");
}

export function missing() {
  return query("SELECT id FROM relation_not_in_catalog");
}

export function collision(id: string) {
  return query("SELECT id FROM active_accounts WHERE id = $1", [id]);
}
