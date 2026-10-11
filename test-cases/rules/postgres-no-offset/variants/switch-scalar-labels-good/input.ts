import { query } from "@example/db";
const statement = "SELECT 1";
let gate = false;
switch (2) {
  case (gate = true, 1): break;
  case 2:
    query("SELECT 2");
    query(statement);
    break;
}
