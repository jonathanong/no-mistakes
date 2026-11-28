const spec = "node:fs";
const loaded = require(spec);
loaded.writeFileSync("/tmp/a.txt", "a");
loaded.lstatSync("/tmp/a.txt");
loaded.access("/tmp/a.txt", () => {});
loaded.chmod("/tmp/a.txt", 0o644, () => {});
loaded.readdir("/tmp");
loaded.createWriteStream("/tmp/a.txt");
