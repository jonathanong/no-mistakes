import * as fs from "node:fs";
const method = "writeFileSync";
const { [method]: write } = fs;
write("/tmp/result", "");
