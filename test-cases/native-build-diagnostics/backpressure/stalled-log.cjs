const fs = require("node:fs");
const path = require("node:path");
const { Writable } = require("node:stream");

const createWriteStream = fs.createWriteStream;
fs.createWriteStream = function (file, ...args) {
  if (path.resolve(file) !== path.resolve("target/native-build-diagnostics/cargo.log"))
    return createWriteStream.call(this, file, ...args);
  return new Writable({
    highWaterMark: 1,
    write(_chunk, _encoding, _callback) {
      process.stderr.write("fixture-log-stalled\n");
    },
  });
};
