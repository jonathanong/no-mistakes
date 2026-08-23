"use strict";

const fs = require("node:fs");
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

function decamelizePlanOptions(options = {}) {
  const next = { ...options };
  if (next.planJson != null) {
    next.planJson = loadPlanJson(next.planJson);
  } else if (typeof next.plan === "string") {
    try {
      next.planJson = loadPlanJson(fs.readFileSync(next.plan, "utf8"));
      delete next.plan;
    } catch {
      // Native still loads missing or invalid plan paths.
    }
  }
  return next;
}

function materializeWhyPlan(options = {}) {
  const next = { ...options };
  let document = next.planJson;
  if (document == null && typeof next.plan === "string") {
    try {
      document = JSON.parse(fs.readFileSync(next.plan, "utf8"));
    } catch {
      return next;
    }
  }
  if (document == null) return next;
  const tmp = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "no-mistakes-why-")), "plan.json");
  fs.writeFileSync(tmp, JSON.stringify(loadPlanJson(document)));
  next.plan = tmp;
  delete next.planJson;
  return next;
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
  const input = Buffer.from(JSON.stringify(decamelizePlanOptions(options)));
  return String(await native.testsCommentMarkdown(input));
}

async function testsGraphMermaid(options) {
  const input = Buffer.from(JSON.stringify(decamelizePlanOptions(options)));
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
  return camelizeWhy(await jsonApis.testsWhy(materializeWhyPlan(options)));
}

async function testsGraph(options) {
  return camelizeValue(await jsonApis.testsGraph(decamelizePlanOptions(options)));
}

module.exports = {
  camelizeValue,
  camelizeWhy,
  decamelizePlanOptions,
  materializeWhyPlan,
  testsComment,
  testsGraphMermaid,
  ...jsonApis,
  testsGraph,
  testsImpact,
  testsPlan,
  testsTargets,
  testsWhy,
};
