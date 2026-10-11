import { query } from "@example/db";
let statement = "SELECT id FROM users";
switch (runtimeMode) {
  case "first": statement += " OFFSET 1";
  case query(statement): break;
}
let live = "SELECT id FROM accounts";
switch (runtimeMode) {
  case "first": live += " LIMIT 1";
  case (query(live), "second"):
    live += " OFFSET 2";
    query(live);
    break;
}
