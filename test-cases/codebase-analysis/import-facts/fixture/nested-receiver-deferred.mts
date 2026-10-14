import { primary } from '@vendor/valkey';
function replacement() {}
const labeledBox = { client: primary };
outer: {
  function resetLabeled() {
    labeledBox.client = {};
    labeledBox.client.get = replacement;
  }
}
const conditionalBox = { client: primary };
if (flag) {
  const resetConditional = () => {
    conditionalBox.client = {};
    conditionalBox.client.get = replacement;
  };
}
const caughtBox = { client: primary };
try {
  const holder = { reset() {
    caughtBox.client = {};
    caughtBox.client.get = replacement;
  } };
} catch {}
const guardedBox = { client: primary };
outerGuard: {
  function resetGuarded() {
    if (flag) guardedBox.client = {};
    guardedBox.client.get = replacement;
  }
}
const defaultBox = { client: primary };
const defaults = { reset: () => {
  const { value = (defaultBox.client = {}) } = source;
  defaultBox.client.get = replacement;
} };
const logicalBox = { client: primary };
flag && (holder = function resetLogical() {
  logicalBox.client = {};
  logicalBox.client.get = replacement;
});
