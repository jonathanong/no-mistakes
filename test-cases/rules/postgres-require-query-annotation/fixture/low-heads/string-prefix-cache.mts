import { read } from "@bench/db";
function accepted(suffix: string) {
  let sql = "/* accepted */ SELECT 1";
  sql += ` ${suffix}`;
  read(sql);
  read(sql);
  read(sql);
}
function rejected(suffix: string) {
  let sql = "/* rejected */ SELECT 1";
  sql += suffix;
  read(sql);
  read(sql);
  read(sql);
}
