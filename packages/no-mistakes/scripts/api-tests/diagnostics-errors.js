const { assert, test, join, packageRoot } = require("./context");
const { fixtures, withApi } = require("./diagnostics-context");
const { callJson } = require(join(packageRoot, "diagnostics.js"));
const MARKER = "\nno-mistakes-diagnostics-json: ";

test("errors preserve identity, code and authoritative details while exposing diagnostics", async () => {
  const { message, details } = fixtures.error;
  const error = new Error(
    message +
      "\nno-mistakes-error-json: " +
      JSON.stringify(details) +
      MARKER +
      JSON.stringify(fixtures.diagnostics),
  );
  await withApi(
    {
      checkJson: async () => {
        throw error;
      },
    },
    async (api) => {
      await assert.rejects(api.collectDiagnostics("check", {}), (received) => {
        assert.equal(received, error);
        assert.equal(received.message, message);
        assert.equal(received.code, "NO_MISTAKES_DEADLINE");
        assert.deepEqual(received.details, details);
        assert.deepEqual(received.diagnostics, fixtures.diagnostics);
        return true;
      });
    },
  );
});

test("malformed diagnostic metadata cannot hide an authoritative invocation failure", async () => {
  const { message, details } = fixtures.error;
  const error = new Error(
    message + "\nno-mistakes-error-json: " + JSON.stringify(details) + MARKER + "{invalid",
  );
  await withApi(
    {
      checkJson: async () => {
        throw error;
      },
    },
    async (api) => {
      await assert.rejects(api.collectDiagnostics("check", {}), (received) => {
        assert.equal(received, error);
        assert.equal(received.code, "NO_MISTAKES_DEADLINE");
        assert.equal(received.diagnostics, undefined);
        return true;
      });
    },
  );
});

test("validation errors without diagnostics remain authoritative", async () => {
  const error = new Error("invalid request");
  error.code = "INVALID_REQUEST";
  error.details = { field: "root" };
  await withApi(
    {
      checkJson: async () => {
        throw error;
      },
    },
    async (api) => {
      await assert.rejects(api.collectDiagnostics("check", {}), (received) => {
        assert.equal(received, error);
        assert.equal(received.code, "INVALID_REQUEST");
        assert.deepEqual(received.details, { field: "root" });
        assert.equal(received.diagnostics, undefined);
        return true;
      });
    },
  );
});

test("collection rejects utilities, raw bindings and unknown operations", async () => {
  await withApi({}, async (api) => {
    for (const operation of [
      "collectDiagnostics",
      "version",
      "diagnosticsSnapshot",
      "writePlanningImpactArtifacts",
      "createWorkflowTopologyIndex",
      "testsComment",
      "testsGraphMermaid",
      "checkJson",
      "unknown",
      "__proto__",
      "toString",
      null,
    ]) {
      await assert.rejects(api.collectDiagnostics(operation), /unsupported diagnostics operation/);
    }
  });
});

test("ordinary errors preserve diagnostic marker text without attaching metadata", async () => {
  for (const suffix of [JSON.stringify(fixtures.diagnostics), "{invalid"]) {
    const message = "ordinary failure" + MARKER + suffix;
    const error = new Error(message);
    await assert.rejects(
      callJson(async () => {
        throw error;
      }, {}),
      (received) => {
        assert.equal(received, error);
        assert.equal(received.message, message);
        assert.equal(Object.hasOwn(received, "diagnostics"), false);
        return true;
      },
    );
  }
});
