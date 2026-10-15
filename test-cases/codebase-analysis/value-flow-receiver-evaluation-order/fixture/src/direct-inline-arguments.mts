import { directLive, directSaved, directOnce, directSibling, directNested } from "@vendor/valkey";
function replacement() {}

const liveAlias = directLive;
function runDirect() {
  (function takeLive(fn) {
    fn("key");
  })(liveAlias.get);
  directLive.get = replacement;
}
for (const value of [0, 1]) runDirect(value);

// A module method copy stays independent of every later live receiver read.
const copied = directSaved.get;
function runCopied() {
  (function takeCopied(fn) {
    fn("key");
  })(copied);
  directSaved.get = replacement;
}
for (const value of [0, 1]) runCopied(value);

// A one-shot read still precedes the write in that same invocation.
function runOnce() {
  (function takeOnce(fn) {
    fn("key");
  })(directOnce.get);
  directOnce.get = replacement;
}
runOnce();

function runSibling() {
  (function takeSibling(fn) {
    fn("key");
  })(directSibling.set);
  directSibling.get = replacement;
}
for (const value of [0, 1]) runSibling(value);

function runNested() {
  (function takeNested(fn) {
    fn("key");
  })(directNested.get);
  directNested.get = replacement;
}
function forwardNested() {
  runNested();
}
for (const value of [0, 1]) forwardNested(value);
