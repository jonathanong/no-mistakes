const spec = "node:child_process";
const cp = require(spec);
cp.execFile("/tmp/result", []);
cp.spawn("/tmp/result", []);
cp.exec("/tmp/result");
cp.execFileSync("/tmp/result", []);
cp.unref("/tmp/result");
