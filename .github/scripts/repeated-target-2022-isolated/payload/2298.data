import {
  repeatedLocal,
  beforeLoop,
  earlierAlias,
  localSibling,
  moduleRepeated,
  moduleSaved,
  deferredCopy,
  unrelatedCopy,
  nestedLoop,
  loopInitializer,
  unrelatedWriter,
} from "@vendor/valkey";
function replacement() {}
function runRepeated() {
  for (const value of [0, 1]) {
    const saved = repeatedLocal.get;
    (function takeRepeated(fn) {
      fn(value);
    })(saved);
    repeatedLocal.get = replacement;
  }
}
runRepeated();
function runBefore() {
  const saved = beforeLoop.get;
  for (const value of [0, 1]) {
    (function takeBefore(fn) {
      fn(value);
    })(saved);
    beforeLoop.get = replacement;
  }
}
runBefore();
// A loop-local alias does not reacquire an earlier copied property.
const earlySaved = earlierAlias.get;
function runAlias() {
  for (const value of [0, 1]) {
    const alias = earlySaved;
    (function takeAlias(fn) {
      fn(value);
    })(alias);
    earlierAlias.get = replacement;
  }
}
runAlias();
function runSibling() {
  for (const value of [0, 1]) {
    const saved = localSibling.set;
    (function takeSibling(fn) {
      fn(value);
    })(saved);
    localSibling.get = replacement;
  }
}
runSibling();
for (const value of [0, 1]) {
  const saved = moduleRepeated.get;
  (function takeModuleRepeated(fn) {
    fn(value);
  })(saved);
  moduleRepeated.get = replacement;
}
const savedModule = moduleSaved.get;
for (const value of [0, 1]) {
  const alias = savedModule;
  (function takeModuleSavedAlias(fn) {
    fn(value);
  })(alias);
  moduleSaved.get = replacement;
}
// Creating a callable in a loop does not repeat its one-shot local acquisition.
let deferred;
for (const value of [0, 1]) {
  deferred = function oneShot() {
    const saved = deferredCopy.get;
    (function takeDeferred(fn) {
      fn(value);
    })(saved);
    deferredCopy.get = replacement;
  };
}
deferred();
function runUnrelated() {
  for (const value of [0, 1]) {
    const saved = unrelatedCopy.get;
    (function takeUnrelated(fn) {
      fn(value);
    })(saved);
    unrelatedWriter.get = replacement;
  }
}
runUnrelated();
function runNested() {
  for (const outer of [0, 1]) {
    for (const value of [0, 1]) {
      const saved = nestedLoop.get;
      (function takeNested(fn) {
        fn(outer, value);
      })(saved);
      nestedLoop.get = replacement;
    }
  }
}
runNested();
// A for initializer executes once, outside the repeated test/update/body.
function runInitializer() {
  for (const saved = loopInitializer.get; check();) {
    (function takeInitializer(fn) {
      fn("key");
    })(saved);
    loopInitializer.get = replacement;
  }
}
function check() {
  return true;
}
runInitializer();
