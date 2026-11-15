import { query, sql } from "@example/db";

export function typedRead(table: "users" | "posts") {
  return query(sql`/* typedRead */ SELECT 1 FROM `.append(table).append(sql` WHERE id = $1`));
}

function tableFor(kind: "user" | "post") {
  switch (kind) {
    case "user":
      return "users";
    case "post":
      return "posts";
  }
}

export function switchRead(kind: "user" | "post") {
  return query(
    sql`/* switchRead */ SELECT 1 FROM `.append(tableFor(kind)).append(sql` WHERE id = $1`),
  );
}
