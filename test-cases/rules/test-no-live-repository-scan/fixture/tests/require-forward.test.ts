// A function visited before the load still reads the initialized module at call time.
function live() {
  return fs.readdirSync("src");
}
function bounded() {
  return fs.readdirSync("fixtures");
}
const fs = require("node:fs");
live();
bounded();
