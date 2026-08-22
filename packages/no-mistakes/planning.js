"use strict";

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

function camelizeValue(value) {
  if (Array.isArray(value)) return value.map(camelizeValue);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, nested]) => [camelizeKey(key), camelizeValue(nested)]),
    );
  }
  return value;
}

async function testsComment(options) {
  const input = Buffer.from(JSON.stringify(options || {}));
  return String(await native.testsCommentMarkdown(input));
}

async function testsGraphMermaid(options) {
  const input = Buffer.from(JSON.stringify(options || {}));
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

module.exports = {
  camelizeValue,
  testsComment,
  testsGraphMermaid,
  ...jsonApis,
  testsImpact,
  testsPlan,
};
