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

const instanceBox = { client: primary };
void class { field = (instanceBox.client = {}); };
instanceBox.client.get = replacement;
const accessorBox = { client: primary };
void class { accessor field = (accessorBox.client = {}); };
accessorBox.client.get = replacement;
const staticBox = { client: primary };
void class { static field = (staticBox.client = {}); };
staticBox.client.get = replacement;
const staticAccessorBox = { client: primary };
void class { static accessor field = (staticAccessorBox.client = {}); };
staticAccessorBox.client.get = replacement;
const namedInstanceBox = { client: primary };
class Deferred { field = (namedInstanceBox.client = {}); }
namedInstanceBox.client.get = replacement;
const computedKeyBox = { client: primary };
void class { [(computedKeyBox.client = {}, 'field')] = 0; };
computedKeyBox.client.get = replacement;
