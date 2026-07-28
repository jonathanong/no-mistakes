"use strict";

const native = require("./bin/no-mistakes.node");

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

async function testsComment(options) {
  return native.testsCommentMarkdown(JSON.stringify(options || {}));
}

async function testsGraphMermaid(options) {
  return native.testsGraphMermaid(JSON.stringify(options || {}));
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

module.exports = {
  testsComment,
  testsGraphMermaid,
  ...jsonApis,
};
