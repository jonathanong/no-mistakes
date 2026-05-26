"use strict";

const native = require("./bin/no-mistakes.node");

async function callJson(fn, options) {
  return JSON.parse(await fn(JSON.stringify(options || {})));
}

async function dependencies(options) {
  return callJson(native.dependenciesJson, options);
}

async function dependents(options) {
  return callJson(native.dependentsJson, options);
}

async function related(options) {
  return callJson(native.relatedJson, options);
}

async function symbols(options) {
  return callJson(native.symbolsJson, options);
}

async function fetches(options) {
  return callJson(native.fetchesJson, options);
}

async function check(options) {
  return callJson(native.checkJson, options);
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

async function playwrightCheck(options) {
  return callJson(native.playwrightCheckJson, options);
}

async function playwrightEdges(options) {
  return callJson(native.playwrightEdgesJson, options);
}

async function playwrightRelated(options) {
  return callJson(native.playwrightRelatedJson, options);
}

async function playwrightTests(options) {
  return callJson(native.playwrightTestsJson, options);
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

async function reactAnalyze(options) {
  return callJson(native.reactAnalyzeJson, options);
}

async function reactCheck(options) {
  return callJson(native.reactCheckJson, options);
}

async function version() {
  return native.version();
}

module.exports = {
  check,
  dependencies,
  dependents,
  fetches,
  playwrightCheck,
  playwrightEdges,
  playwrightRelated,
  playwrightTests,
  queues,
  queueCheck,
  queueEdges,
  queueRelated,
  reactAnalyze,
  reactCheck,
  related,
  serverRouteEdges,
  serverRouteList,
  serverRouteRelated,
  serverRoutes,
  symbols,
  testsComment,
  testsGraph,
  testsGraphMermaid,
  testsImpact,
  testsPlan,
  testsWhy,
  version,
};
