import { query } from "@example/db";
let good = flag ? "SELECT 1" : "SELECT 2";
let bad = flag ? "SELECT 3 OFFSET 1" : "SELECT 4";
switch (mode) {
  case 0: break;
  case 1: break;
  case 2: break;
  case 3: break;
  case 4: break;
  case 5: break;
  case 6: break;
  case 7: break;
}
query(good);
query(bad);
