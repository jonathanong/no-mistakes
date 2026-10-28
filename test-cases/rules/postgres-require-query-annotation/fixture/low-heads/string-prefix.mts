import { read } from "@bench/db";
import { read as foreign } from "@other/db";
declare const flag: boolean, suffix: string, count: number;
declare function consume(value: unknown): void;
declare function tag(parts: TemplateStringsArray): unknown;

function branches(params: unknown[]) {
  let sql = `/* list/search */ SELECT id FROM lists WHERE owner = $1`;
  if (flag) sql += ` AND visible = true`;
  if (suffix) {
    params.push(suffix);
    sql += ` AND id < $${params.length}`;
  } else sql += " AND active = true";
  sql += ` ORDER BY id DESC LIMIT $${params.length + 1}`;
  read(sql, params); // low: true
  read(sql, params); // low: true
}
function interpolatedInitial() {
  let sql = `/* interpolated */ SELECT ${count}`;
  sql += ` ${suffix}`;
  read(sql); // low: true
}
function nestedLexicalBlock() {
  { { { { let sql = "/* nested */ SELECT 1";
    sql += " LIMIT 1";
    read(sql); // low: true
  } } } }
}
function partialTransaction() {
  // EOF BEGIN is exempt, but BEGINNING is not: the initial head must survive continuation.
  let sql = "BEGIN";
  sql += "NING";
  read(sql); // low: false
}
function partialComment() {
  let sql = "/*";
  sql += " later */ SELECT 1";
  read(sql); // low: false
}
function missingHead() {
  let sql = "SELECT 1";
  sql += " /* too late */";
  read(sql); // low: false
}
function subtract() {
  let sql = "/* subtract */ SELECT 1";
  sql -= "1";
  read(sql); // low: false
}
function reset() {
  let sql = "/* reset */ SELECT 1";
  sql += " LIMIT 1";
  sql = suffix;
  read(sql); // low: false
}
function numericAppend() {
  let sql = "/* numeric */ SELECT 1";
  sql += count;
  read(sql); // low: false
}
function unknownAppend() {
  let sql = "/* unknown */ SELECT 1";
  sql += suffix;
  read(sql); // low: false
}
function concatAppend() {
  let sql = "/* concat */ SELECT 1";
  sql += " LIMIT " + count;
  read(sql); // low: false
}
function increment() {
  let sql = "/* increment */ SELECT 1";
  sql += " LIMIT 1";
  sql++;
  read(sql); // low: false
}
function alias() {
  let sql = "/* alias */ SELECT 1";
  const copy = sql;
  sql += " LIMIT 1";
  consume(copy);
  read(sql); // low: false
}
function captured() {
  let sql = "/* captured */ SELECT 1";
  const callback = () => { sql += " LIMIT 1"; };
  callback();
  read(sql); // low: false
}
function escapingUse() {
  let sql = "/* foreign */ SELECT 1";
  sql += " LIMIT 1";
  foreign(sql);
  read(sql); // low: false
}
function secondArgument() {
  let sql = "/* second */ SELECT 1";
  sql += " LIMIT 1";
  read("/* other */ SELECT 2", sql); // low: true
  read(sql); // low: false
}
function usedAssignment() {
  let sql = "/* assignment */ SELECT 1";
  consume(sql += " LIMIT 1");
  read(sql); // low: false
}
function reentrantWrite() {
  let sql = "/* reentrant */ SELECT 1";
  sql += ` ${sql = suffix}`;
  read(sql); // low: false
}
function reentrantRead() {
  let sql = "/* reentrantRead */ SELECT 1";
  // The existing direct-literal proof covers the inner read before += completes.
  sql += ` ${read(sql)}`; // low: true
  read(sql); // low: false
}
function loop() {
  let sql = "/* loop */ SELECT 1";
  while (flag) sql += " LIMIT 1";
  read(sql); // low: false
}
function optional() {
  let sql = "/* optional */ SELECT 1";
  sql += " LIMIT 1";
  read?.(sql); // low: false
}
function tagged() {
  let sql = tag`/* tag */ SELECT 1`;
  sql += " LIMIT 1";
  read(sql); // low: false
}
function shadow() {
  let sql = "/* shadow */ SELECT 1";
  sql += " LIMIT 1";
  { let sql = suffix; sql += " LIMIT 2"; read(sql); } // low: false
  read(sql); // low: true
}
function nonStringInitial() {
  let sql = 1;
  sql += "/* numeric head */ SELECT 1";
  read(sql); // low: false
}
