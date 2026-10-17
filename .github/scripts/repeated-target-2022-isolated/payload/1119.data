import { doubleDeferredCopy } from "@vendor/valkey";

function replacement() {}
let deferred;
for (const value of [0, 1]) {
  deferred = function calledTwice() {
    // The first call writes the receiver; the next acquisition sees that write.
    const saved = doubleDeferredCopy.get;
    (function takeTwice(fn) {
      fn(value);
    })(saved);
    doubleDeferredCopy.get = replacement;
  };
}
deferred();
deferred();
