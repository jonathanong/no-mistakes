"use strict";

const native = require("./bin/no-mistakes.node");

async function callJson(fn, options) {
  return JSON.parse(await fn(JSON.stringify(options || {})));
}

async function testsTargets(options) {
  return callJson(native.testsTargetsJson, options);
}

async function testsPlan(options) {
  return callJson(native.testsPlanJson, options);
}

async function testsImpact(options) {
  return callJson(native.testsImpactJson, options);
}

async function testsWhy(options) {
  return callJson(native.testsWhyJson, options);
}

async function testsComment(options) {
  return native.testsCommentMarkdown(JSON.stringify(options || {}));
}

async function testsGraph(options) {
  return callJson(native.testsGraphJson, options);
}

async function testsGraphMermaid(options) {
  return native.testsGraphMermaid(JSON.stringify(options || {}));
}

async function queues(options) {
  return callJson(native.queuesJson, options);
}

async function queueEdges(options) {
  return callJson(native.queueEdgesJson, options);
}

async function queueRelated(options) {
  return callJson(native.queueRelatedJson, options);
}

async function queueCheck(options) {
  return callJson(native.queueCheckJson, options);
}

async function serverRoutes(options) {
  return callJson(native.serverRoutesJson, options);
}

async function serverRouteList(options) {
  return callJson(native.serverRouteListJson, options);
}

async function serverRouteEdges(options) {
  return callJson(native.serverRouteEdgesJson, options);
}

async function serverRouteRelated(options) {
  return callJson(native.serverRouteRelatedJson, options);
}

async function serverContracts(options) {
  return callJson(native.serverContractsJson, options);
}

async function flow(options) {
  return callJson(native.flowJson, options);
}

module.exports = {
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
};
