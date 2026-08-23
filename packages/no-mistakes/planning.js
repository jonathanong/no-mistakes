"use strict";

const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const native = require(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH || "./bin/no-mistakes.node");

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

function camelizeKey(key) {
  return key.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

function decamelizeKey(key) {
  return key.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}

function mapKeys(value, mapKey) {
  if (Array.isArray(value)) return value.map((item) => mapKeys(item, mapKey));
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, nested]) => [mapKey(key), mapKeys(nested, mapKey)]),
    );
  }
  return value;
}

function camelizeValue(value) {
  return mapKeys(value, camelizeKey);
}

function decamelizeValue(value) {
  return mapKeys(value, decamelizeKey);
}

function loadPlanJson(planJson) {
  let parsed = planJson;
  if (typeof parsed === "string") {
    try {
      parsed = JSON.parse(parsed);
    } catch {
      return planJson;
    }
  }
  if (parsed && typeof parsed === "object") {
    return decamelizeValue(parsed);
  }
  return planJson;
}

async function readPlanFile(planPath) {
  try {
    return JSON.parse(await fs.readFile(planPath, "utf8"));
  } catch {
    return undefined;
  }
}

async function decamelizePlanOptions(options = {}) {
  const next = { ...options };
  if (next.planJson != null) {
    next.planJson = loadPlanJson(next.planJson);
  } else if (typeof next.plan === "string") {
    const document = await readPlanFile(next.plan);
    if (document !== undefined) {
      next.planJson = loadPlanJson(document);
      delete next.plan;
    }
  }
  return next;
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
  await fs.writeFile(path.join(generatedDir, "plan.json"), JSON.stringify(loadPlanJson(document)));
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
  testsGraph: "testsGraphJson",
  testsImpact: "testsImpactJson",
  testsPlan: "testsPlanJson",
  testsTargets: "testsTargetsJson",
  testsWhy: "testsWhyJson",
});

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
  prepareWhyPlan,
  removeGeneratedDir,
  testsComment,
  testsGraphMermaid,
  ...jsonApis,
  testsGraph,
  testsImpact,
  testsPlan,
  testsTargets,
  testsWhy,
};
