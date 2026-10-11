"use strict";

const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const { resolveNativePackage } = require("./scripts/native-package");
const native = require(
  process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH || resolveNativePackage().addonPath,
);
const {
  camelizeValue,
  decamelizePlanOptions,
  loadPlanJson,
  readPlanFile,
} = require("./planning-artifact-inputs");
const PLAN_INPUT_REPORTS = new Set(["testsComment", "testsGraph", "testsGraphMermaid"]);

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

async function prepareWhyPlan(options = {}) {
  const next = { ...options };
  let document = next.planJson;
  if (document == null && typeof next.plan === "string") {
    document = await readPlanFile(next.plan);
    if (document === undefined) return { request: next };
  }
  if (document == null) return { request: next };
  const generatedDir = await fs.mkdtemp(path.join(os.tmpdir(), "no-mistakes-why-"));
  try {
    await fs.writeFile(
      path.join(generatedDir, "plan.json"),
      JSON.stringify(loadPlanJson(document)),
    );
  } catch (error) {
    await removeGeneratedDir(generatedDir);
    throw error;
  }
  next.plan = path.join(generatedDir, "plan.json");
  delete next.planJson;
  return { request: next, generatedDir };
}

async function materializeWhyPlan(options = {}) {
  return (await prepareWhyPlan(options)).request;
}

async function removeGeneratedDir(generatedDir) {
  if (!generatedDir) return;
  await fs.rm(generatedDir, { recursive: true, force: true }).catch(() => {});
}

async function prepareAnalyzeProjectReports(reports, generatedDirs) {
  const preparations = reports.map(async (report) => {
    if (report.type === "testsWhy") {
      const prepared = await prepareWhyPlan(report);
      if (prepared.generatedDir) generatedDirs.push(prepared.generatedDir);
      return prepared.request;
    }
    if (report.type === "testsAudit") return await prepareAuditOptions(report);
    return PLAN_INPUT_REPORTS.has(report.type) ? await decamelizePlanOptions(report) : report;
  });
  const settled = await Promise.allSettled(preparations);
  const rejected = settled.find((result) => result.status === "rejected");
  if (rejected) throw rejected.reason;
  return settled.map((result) => result.value);
}

function camelizeWhy(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return camelizeValue(value);
  }
  return Object.fromEntries(
    Object.entries(value).map(([key, nested]) => [key, camelizeValue(nested)]),
  );
}

async function testsComment(options) {
  const input = Buffer.from(JSON.stringify(await decamelizePlanOptions(options)));
  return String(await native.testsCommentMarkdown(input));
}

async function testsGraphMermaid(options) {
  const input = Buffer.from(JSON.stringify(await decamelizePlanOptions(options)));
  return String(await native.testsGraphMermaid(input));
}

const jsonApis = createJsonApis({
  flow: "flowJson",
  queueCheck: "queueCheckJson",
  queueEdges: "queueEdgesJson",
  queueRelated: "queueRelatedJson",
  queues: "queuesJson",
  serverContracts: "serverContractsJson",
  serverRouteEdges: "serverRouteEdgesJson",
  serverRouteList: "serverRouteListJson",
  serverRouteRelated: "serverRouteRelatedJson",
  serverRoutes: "serverRoutesJson",
  testsAudit: "testsAuditJson",
  testsGraph: "testsGraphJson",
  testsImpact: "testsImpactJson",
  testsPlan: "testsPlanJson",
  testsTargets: "testsTargetsJson",
  testsWhy: "testsWhyJson",
});

async function prepareAuditOptions(options = {}) {
  const next = await decamelizePlanOptions(options);
  if (next.observationsJson != null) {
    next.observationsJson = loadPlanJson(next.observationsJson);
  } else if (typeof next.observations === "string") {
    const document = await readPlanFile(next.observations);
    if (document !== undefined) {
      next.observationsJson = loadPlanJson(document);
      delete next.observations;
    }
  }
  return next;
}

async function testsAudit(options) {
  return camelizeValue(await jsonApis.testsAudit(await prepareAuditOptions(options)));
}

async function testsPlan(options) {
  return camelizeValue(await jsonApis.testsPlan(options));
}

async function testsImpact(options) {
  return camelizeValue(await jsonApis.testsImpact(options));
}

async function testsTargets(options) {
  return camelizeValue(await jsonApis.testsTargets(options));
}

async function testsWhy(options) {
  const { request, generatedDir } = await prepareWhyPlan(options);
  try {
    return camelizeWhy(await jsonApis.testsWhy(request));
  } finally {
    await removeGeneratedDir(generatedDir);
  }
}

async function testsGraph(options) {
  return camelizeValue(await jsonApis.testsGraph(await decamelizePlanOptions(options)));
}

module.exports = {
  camelizeValue,
  camelizeWhy,
  decamelizePlanOptions,
  materializeWhyPlan,
  prepareAnalyzeProjectReports,
  prepareWhyPlan,
  removeGeneratedDir,
  testsComment,
  testsGraphMermaid,
  ...jsonApis,
  testsAudit,
  testsGraph,
  testsImpact,
  testsPlan,
  testsTargets,
  testsWhy,
};
