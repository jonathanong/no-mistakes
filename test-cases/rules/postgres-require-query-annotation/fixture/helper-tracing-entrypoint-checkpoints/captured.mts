import { read } from "@app/db";
function outer(value) {
  const initial = () => read(value);
  // A hoisted local factory copies value ownership into a retained frame.
  function make() {
    return function access(next) {
      read(value);
      value = next;
      initial();
    };
  }
  return make();
}
const access = outer("/* captured initial */ SELECT 1");
const modulePrefix = "/* module builder */ SELECT 2";
export function aWrites() {
  access("SELECT 1");
  read(modulePrefix);
}
export function bReadsInitial() {
  access("/* second */ SELECT 3");
  read(modulePrefix);
}
