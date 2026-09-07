"use strict";

const { resolveNativePackage } = require("./scripts/native-package");
const addonPath = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH || resolveNativePackage().addonPath;
const native = require(addonPath);
const planning = require("./planning");
const { writePlanningImpactArtifacts: writeArtifacts } = require("./planning-impact-artifacts");
const { createPlanningArtifactLock } = require("./planning-impact-artifacts-lock");
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
  ciTopologyImpact: "ciTopologyImpactJson",
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

const PLAN_INPUT_REPORTS = new Set(["testsComment", "testsGraph", "testsGraphMermaid"]);
const CAMELIZE_REPORTS = new Set(["testsPlan", "testsImpact", "testsTargets", "testsGraph"]);
const acquirePlanningArtifactLock = createPlanningArtifactLock(native);

async function analyzeProject(options = {}) {
  const request = { ...options };
  const generatedDirs = [];
  try {
    if (Array.isArray(request.reports)) {
      request.reports = await Promise.all(
        request.reports.map(async (report) => {
          if (report.type === "testsWhy") {
            const prepared = await planning.prepareWhyPlan(report);
            if (prepared.generatedDir) generatedDirs.push(prepared.generatedDir);
            return prepared.request;
          }
          return PLAN_INPUT_REPORTS.has(report.type)
            ? await planning.decamelizePlanOptions(report)
            : report;
        }),
      );
    }
    const result = await jsonApis.analyzeProject(request);
    for (const report of result.reports || []) {
      if (report.type === "testsWhy") {
        report.result = planning.camelizeWhy(report.result);
      } else if (CAMELIZE_REPORTS.has(report.type)) {
        report.result = planning.camelizeValue(report.result);
      }
    }
    return result;
  } finally {
    await Promise.all(generatedDirs.map((dir) => planning.removeGeneratedDir(dir)));
  }
}

async function writePlanningImpactArtifacts(options) {
  return writeArtifacts(
    options,
    analyzeProject,
    async (from, to) => {
      if (await native.renameNoReplace(from, to)) return true;
      const error = new Error("output directory path changed during planning artifact generation");
      error.code = "EEXIST";
      throw error;
    },
    acquirePlanningArtifactLock,
  );
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
    []
      .concat((options && options.workflows) || [])
      .map(String)
      .sort(),
  );
  const identity = `${root}\0${configPath}\0`;
  const key = `${identity}${mtime}\0${workflows}`;
  const stale = [];
  for (const memoKey of topologyMemo.keys()) {
    if (!memoKey.startsWith(identity)) continue;
    const memoMtime = memoKey.slice(identity.length).split("\0")[0];
    if (memoMtime !== String(mtime)) stale.push(memoKey);
  }
  for (const memoKey of stale) topologyMemo.delete(memoKey);
  const cached = topologyMemo.get(key);
  if (cached) return cached.then((value) => structuredClone(value));
  const pending = jsonApis.ciTopology({ ...options, root }).catch((error) => {
    topologyMemo.delete(key);
    throw error;
  });
  topologyMemo.set(key, pending);
  return pending.then((value) => structuredClone(value));
}

async function version() {
  return native.version();
}

const {
  flow,
  queueCheck,
  queueEdges,
  queueRelated,
  queues,
  serverContracts,
  serverRouteEdges,
  serverRouteList,
  serverRouteRelated,
  serverRoutes,
  testsComment,
  testsGraph,
  testsGraphMermaid,
  testsImpact,
  testsPlan,
  testsTargets,
  testsWhy,
} = planning;

Object.assign(module.exports, jsonApis, {
  analyzeProject,
  ciTopology,
  createWorkflowTopologyIndex,
  version,
  writePlanningImpactArtifacts,
  flow,
  queueCheck,
  queueEdges,
  queueRelated,
  queues,
  serverContracts,
  serverRouteEdges,
  serverRouteList,
  serverRouteRelated,
  serverRoutes,
  testsComment,
  testsGraph,
  testsGraphMermaid,
  testsImpact,
  testsPlan,
  testsTargets,
  testsWhy,
});
