import { query } from "@data-stores/psql";

function f1() {
  return f2();
}
function f2() {
  return f3();
}
function f3() {
  return f4();
}
function f4() {
  return f5();
}
function f5() {
  return f6();
}
function f6() {
  return f7();
}
function f7() {
  return f8();
}
function f8() {
  return f9();
}
function f9() {
  return f10();
}
function f10() {
  return f11();
}
function f11() {
  return f12();
}
function f12() {
  return "SELECT 1";
}

const sql = f1();

export function load() {
  return query(sql);
}
