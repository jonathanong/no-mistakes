"use strict";

const native = require("./bin/no-mistakes.node");
const planning = require("./planning");
const { createWorkflowTopologyIndex } = require("./workflow-topology-index");

async function callJson(fn, options) {
  return JSON.parse(await fn(JSON.stringify(options || {})));
}

function createJsonApis(descriptors) {
  return Object.fromEntries(
    Object.entries(descriptors).map(([apiName, nativeName]) => [
      apiName,
      async (options) => callJson(native[nativeName], options),
    ]),
  );
}

const jsonApis = createJsonApis({
  analyzeProject: "analyzeProjectJson",
  callSites: "callSitesJson",
  check: "checkJson",
  ciEnv: "ciEnvJson",
  ciImpact: "ciImpactJson",
  ciTopology: "ciTopologyJson",
  dataPw: "dataPwJson",
  deadExports: "deadExportsJson",
  dependencies: "dependenciesJson",
  dependents: "dependentsJson",
  effects: "effectsJson",
  exportsOf: "exportsOfJson",
  fetches: "fetchesJson",
  impactedChecks: "impactedChecksJson",
  importUsages: "importUsagesJson",
  importers: "importersJson",
  infraOutputs: "infraOutputsJson",
  infraResourceRefs: "infraResourceRefsJson",
  infraTestFor: "infraTestForJson",
  lockfileDiff: "lockfileDiffJson",
  playwrightCheck: "playwrightCheckJson",
  playwrightEdges: "playwrightEdgesJson",
  playwrightRelated: "playwrightRelatedJson",
  playwrightTests: "playwrightTestsJson",
  reactAnalyze: "reactAnalyzeJson",
  reactCheck: "reactCheckJson",
  reactUsages: "reactUsagesJson",
  registryExtension: "registryExtensionJson",
  related: "relatedJson",
  resolveCheck: "resolveCheckJson",
  rscCallers: "rscCallersJson",
  swiftImporters: "swiftImportersJson",
  swiftTestTargets: "swiftTestTargetsJson",
  symbols: "symbolsJson",
});

async function version() {
  return native.version();
}

module.exports = {
  createWorkflowTopologyIndex,
  version,
  ...jsonApis,
  ...planning,
};
