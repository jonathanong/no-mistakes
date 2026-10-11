import { query } from "@example/db";
function returns(flag) {
  let q = "SELECT 1";
  if (flag) { q += " LIMIT 1"; return; }
  else q += " OFFSET 1";
  query(q);
}
function throws(flag) {
  let q = "SELECT 2";
  if (flag) { q += " LIMIT 1"; throw new Error(); }
  else q += " OFFSET 1";
  query(q);
}
function conditionalBreak(mode, flag) {
  let q = "SELECT 3";
  switch (mode) {
    case "a":
      q += " /* a */";
      if (flag) break;
      q += " /* after */";
      break;
    default: q += " /* default */";
  }
  query(q);
}
