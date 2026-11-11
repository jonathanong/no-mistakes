const { Writable } = require("node:stream");

class Capture extends Writable {
  constructor({ highWaterMark = 4096, hold = false } = {}) {
    super({ highWaterMark });
    this.parts = [];
    this.callbacks = [];
    this.hold = hold;
    this.peakWritableLength = 0;
  }

  write(chunk, encoding, callback) {
    const accepted = super.write(chunk, encoding, callback);
    this.peakWritableLength = Math.max(this.peakWritableLength, this.writableLength);
    return accepted;
  }

  _write(chunk, _encoding, callback) {
    this.parts.push(Buffer.from(chunk));
    if (this.hold) this.callbacks.push(callback);
    else setImmediate(callback);
  }

  get bytes() {
    return Buffer.concat(this.parts);
  }

  release() {
    this.hold = false;
    while (this.callbacks.length) this.callbacks.shift()();
  }

  fail(error) {
    const callback = this.callbacks.shift();
    if (!callback) throw new Error("no held write is pending");
    callback(error);
  }
}

module.exports = { Capture };
