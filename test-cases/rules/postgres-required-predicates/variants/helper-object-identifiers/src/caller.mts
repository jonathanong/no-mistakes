import sql from "sql-template-strings";
import { read } from "@example/db";
import { buildFollowedCte } from "./cte.mts";

export async function followedFeed(userId: string, feedType: string, cursorId?: string) {
  const followed = buildFollowedCte(userId, {
    relationTable: "relation_follow",
    idColumn: "user_id",
    cteAlias: "followed_users",
  });
  const query = sql``
    .append(followed)
    .append(sql` SELECT v.id FROM view_active_plans v JOIN followed_users f ON f.user_id = v.owner_id WHERE v.tier = 'a'`);
  if (feedType === "mutual") {
    query.append(sql` AND v.mutual = TRUE`);
  }
  if (cursorId) {
    query.append(sql` AND v.id < ${cursorId}`);
  }
  return read(sql`/* feed */ WITH `.append(query));
}

// Each property below makes the identifier text unknown, so each stays opaque.

export async function propertyIsNotLiteral(userId: string, column: string) {
  const followed = buildFollowedCte(userId, {
    relationTable: "relation_follow",
    idColumn: column,
    cteAlias: "followed_users",
  });
  return read(sql`WITH `.append(followed).append(sql` SELECT 1`));
}

export async function propertyIsMissing(userId: string) {
  const followed = buildFollowedCte(userId, {
    relationTable: "relation_follow",
    cteAlias: "followed_users",
  });
  return read(sql`WITH `.append(followed).append(sql` SELECT 1`));
}

export async function optionsAreSpread(userId: string, shared: object) {
  const followed = buildFollowedCte(userId, { ...shared, cteAlias: "followed_users" });
  return read(sql`WITH `.append(followed).append(sql` SELECT 1`));
}
