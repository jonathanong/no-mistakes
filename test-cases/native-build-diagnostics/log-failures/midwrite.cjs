const fs = require("node:fs");
const path = require("node:path");

const createWriteStream = fs.createWriteStream;
fs.createWriteStream = function (file, ...args) {
  const stream = createWriteStream.call(this, file, ...args);
  if (path.resolve(file) !== path.resolve("target/native-build-diagnostics/cargo.log"))
    return stream;
  const write = stream._write;
  stream._write = function (chunk, encoding, callback) {
    write.call(
      this,
      chunk.subarray(0, Math.max(1, Math.floor(chunk.length / 2))),
      encoding,
      (error) => {
        setImmediate(() =>
          callback(error || Object.assign(new Error("fixture disk full"), { code: "ENOSPC" })),
        );
      },
    );
  };
  stream._writev = function (chunks, callback) {
    this._write(Buffer.concat(chunks.map(({ chunk }) => chunk)), "buffer", callback);
  };
  return stream;
};
