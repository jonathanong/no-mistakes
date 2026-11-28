import * as fs from "node:fs";
fs.writeFileSync("/tmp/a.txt", "a");
const { ["writeFileSync" as const]: write } = fs;
write("/tmp/b.txt", "b");
const loaded = (require as NodeRequire)("node:fs");
loaded.writeFileSync("/tmp/c.txt", "c");
