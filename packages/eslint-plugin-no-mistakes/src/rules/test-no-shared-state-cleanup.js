"use strict";

function createCleanupTracker(options = {}) {
  const pathsBySuite = new Map();
  const suiteStack = [];
  const serialStack = [];
  const serialSuites = new Set();
  const pendingBeforeAll = new Map();
  let fileSerial = false;
  let activeSuiteKey, pendingBeforeAllKey, replaySuiteKey;
  let nextSuiteId = 0;
  const ao = options.allowBeforeAllAssignments;
  const sKinds = new Set(ao ? ["per-test", "before-once"] : ["per-test"]);

  function currentSuiteKey() {
    return replaySuiteKey ?? suiteStack.join("/");
  }

  function has(path, suiteKey) {
    if (!path) return false;
    for (const [cleanupSuiteKey, paths] of pathsBySuite) {
      if (!paths.has(path)) continue;
      if (!cleanupSuiteKey || suiteKey === cleanupSuiteKey) return true;
      if (suiteKey.startsWith(`${cleanupSuiteKey}/`)) return true;
    }
    return false;
  }

  function suiteIsSerial(suiteKey) {
    return [...serialSuites].some((serialKey) => {
      return suiteKey === serialKey || suiteKey.startsWith(`${serialKey}/`);
    });
  }

  function addPath(map, suiteKey, path) {
    const paths = map.get(suiteKey) ?? new Set();
    map.set(suiteKey, paths.add(path));
  }

  function promoteBeforeAll(suiteKey) {
    for (const [pendingSuiteKey, paths] of pendingBeforeAll) {
      if (suiteKey && pendingSuiteKey !== suiteKey && !pendingSuiteKey.startsWith(`${suiteKey}/`)) {
        continue;
      }
      for (const path of paths) addPath(pathsBySuite, pendingSuiteKey, path);
    }
  }

  return {
    beginSetup(kind, suiteKey = currentSuiteKey()) {
      const serialBeforeAll =
        kind === "before-once" &&
        (fileSerial || serialStack.includes(true) || suiteIsSerial(suiteKey));
      activeSuiteKey = sKinds.has(kind) || serialBeforeAll ? suiteKey : undefined;
      pendingBeforeAllKey =
        kind === "before-once" && activeSuiteKey === undefined ? suiteKey : undefined;
    },
    clearReplaySuite() {
      replaySuiteKey = undefined;
    },
    currentSuiteKey,
    endSetup() {
      activeSuiteKey = undefined;
      pendingBeforeAllKey = undefined;
    },
    enterSuite(serial = false) {
      const key = String(nextSuiteId++);
      suiteStack.push(key);
      serialStack.push(serial);
      if (serial) serialSuites.add(currentSuiteKey());
    },
    exitSuite() {
      suiteStack.pop();
      serialStack.pop();
    },
    has,
    markCurrentSuiteSerial() {
      if (serialStack.length > 0) {
        serialStack[serialStack.length - 1] = true;
        const suiteKey = currentSuiteKey();
        serialSuites.add(suiteKey);
        promoteBeforeAll(suiteKey);
      } else {
        fileSerial = true;
        promoteBeforeAll();
      }
    },
    remember(path) {
      if (!path || activeSuiteKey === undefined) return;
      addPath(pathsBySuite, activeSuiteKey, path);
    },
    rememberPendingBeforeAll(path) {
      if (!path || pendingBeforeAllKey === undefined) return;
      addPath(pendingBeforeAll, pendingBeforeAllKey, path);
    },
    setReplaySuite(suiteKey) {
      replaySuiteKey = suiteKey;
    },
  };
}

module.exports = { createCleanupTracker };
