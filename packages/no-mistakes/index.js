"use strict";

// CI real-addon tests point this at the freshly compiled cdylib without
// overwriting the package's checked-in install placeholder.
const native = require(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH || "./bin/no-mistakes.node");
const planning = require("./planning");
const { createWorkflowTopologyIndex } = require("./workflow-topology-index");
const fs = require("node:fs");
const path = require("node:path");

async function callJson(fn, options) {
  const input = Buffer.from(JSON.stringify(options || {}));
  return JSON.parse(await fn(input));
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
  resolveConfig: "resolveConfigJson",
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
  validateMermaidMarkdown: "validateMermaidMarkdownJson",
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

const DOCUMENT_REPORTS = new Set(["testsComment", "testsGraph", "testsGraphMermaid"]);

async function analyzeProject(options = {}) {
  const request = { ...options };
  if (Array.isArray(request.reports)) {
    request.reports = request.reports.map((report) =>
      DOCUMENT_REPORTS.has(report.type) ? planning.decamelizePlanOptions(report) : report,
    );
  }
  const result = await jsonApis.analyzeProject(request);
  for (const report of result.reports || []) {
    if (report.type === "testsPlan" || report.type === "testsImpact") {
      report.result = planning.camelizeValue(report.result);
    }
  }
  return result;
}

const topologyMemo = new Map();

async function ciTopology(options) {
  const root = path.resolve((options && options.root) || process.cwd());
  const configPath = path.resolve(root, (options && options.config) || ".no-mistakes.yml");
  let mtime = 0;
  try {
    mtime = fs.statSync(configPath).mtimeMs;
  } catch {
    mtime = 0;
  }
  const workflows = JSON.stringify(
    [...((options && options.workflows) || [])].map(String).sort(),
  );
  const identity = `${root}\0${configPath}\0`;
  const key = `${identity}${mtime}\0${workflows}`;
  for (const memoKey of [...topologyMemo.keys()]) {
    if (!memoKey.startsWith(identity)) continue;
    const memoMtime = memoKey.slice(identity.length).split("\0")[0];
    if (memoMtime !== String(mtime)) topologyMemo.delete(memoKey);
  }
  const cached = topologyMemo.get(key);
  if (cached) return cached.then((value) => structuredClone(value));
  const pending = jsonApis.ciTopology(options).catch((error) => {
    topologyMemo.delete(key);
    throw error;
  });
  topologyMemo.set(key, pending);
  return pending.then((value) => structuredClone(value));
}

async function version() {
  return native.version();
}

module.exports.createWorkflowTopologyIndex = createWorkflowTopologyIndex;
module.exports.version = version;
module.exports.analyzeProject = analyzeProject;
module.exports.callSites = jsonApis.callSites;
module.exports.check = jsonApis.check;
module.exports.resolveConfig = jsonApis.resolveConfig;
module.exports.ciEnv = jsonApis.ciEnv;
module.exports.ciImpact = jsonApis.ciImpact;
module.exports.ciTopology = ciTopology;
module.exports.dataPw = jsonApis.dataPw;
module.exports.deadExports = jsonApis.deadExports;
module.exports.dependencies = jsonApis.dependencies;
module.exports.dependents = jsonApis.dependents;
module.exports.effects = jsonApis.effects;
module.exports.exportsOf = jsonApis.exportsOf;
module.exports.fetches = jsonApis.fetches;
module.exports.impactedChecks = jsonApis.impactedChecks;
module.exports.importUsages = jsonApis.importUsages;
module.exports.importers = jsonApis.importers;
module.exports.infraOutputs = jsonApis.infraOutputs;
module.exports.infraResourceRefs = jsonApis.infraResourceRefs;
module.exports.infraTestFor = jsonApis.infraTestFor;
module.exports.lockfileDiff = jsonApis.lockfileDiff;
module.exports.validateMermaidMarkdown = jsonApis.validateMermaidMarkdown;
module.exports.playwrightCheck = jsonApis.playwrightCheck;
module.exports.playwrightEdges = jsonApis.playwrightEdges;
module.exports.playwrightRelated = jsonApis.playwrightRelated;
module.exports.playwrightTests = jsonApis.playwrightTests;
module.exports.reactAnalyze = jsonApis.reactAnalyze;
module.exports.reactCheck = jsonApis.reactCheck;
module.exports.reactUsages = jsonApis.reactUsages;
module.exports.registryExtension = jsonApis.registryExtension;
module.exports.related = jsonApis.related;
module.exports.resolveCheck = jsonApis.resolveCheck;
module.exports.rscCallers = jsonApis.rscCallers;
module.exports.swiftImporters = jsonApis.swiftImporters;
module.exports.swiftTestTargets = jsonApis.swiftTestTargets;
module.exports.symbols = jsonApis.symbols;
module.exports.testsComment = planning.testsComment;
module.exports.testsGraphMermaid = planning.testsGraphMermaid;
module.exports.flow = planning.flow;
module.exports.queueCheck = planning.queueCheck;
module.exports.queueEdges = planning.queueEdges;
module.exports.queueRelated = planning.queueRelated;
module.exports.queues = planning.queues;
module.exports.serverContracts = planning.serverContracts;
module.exports.serverRouteEdges = planning.serverRouteEdges;
module.exports.serverRouteList = planning.serverRouteList;
module.exports.serverRouteRelated = planning.serverRouteRelated;
module.exports.serverRoutes = planning.serverRoutes;
module.exports.testsGraph = planning.testsGraph;
module.exports.testsImpact = planning.testsImpact;
module.exports.testsPlan = planning.testsPlan;
module.exports.testsTargets = planning.testsTargets;
module.exports.testsWhy = planning.testsWhy;
