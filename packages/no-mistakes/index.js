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

async function analyzeProject(options) {
  return callJson(native.analyzeProjectJson, options);
}

async function symbols(options) {
  return callJson(native.symbolsJson, options);
}

async function importers(options) {
  return callJson(native.importersJson, options);
}

async function exportsOf(options) {
  return callJson(native.exportsOfJson, options);
}

async function deadExports(options) {
  return callJson(native.deadExportsJson, options);
}

async function callSites(options) {
  return callJson(native.callSitesJson, options);
}

async function resolveCheck(options) {
  return callJson(native.resolveCheckJson, options);
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

async function reactUsages(options) {
  return callJson(native.reactUsagesJson, options);
}

async function dataPw(options) {
  return callJson(native.dataPwJson, options);
}

async function effects(options) {
  return callJson(native.effectsJson, options);
}

async function rscCallers(options) {
  return callJson(native.rscCallersJson, options);
}

async function registryExtension(options) {
  return callJson(native.registryExtensionJson, options);
}

async function lockfileDiff(options) {
  return callJson(native.lockfileDiffJson, options);
}

async function infraResourceRefs(options) {
  return callJson(native.infraResourceRefsJson, options);
}

async function infraOutputs(options) {
  return callJson(native.infraOutputsJson, options);
}

async function infraTestFor(options) {
  return callJson(native.infraTestForJson, options);
}

async function swiftImporters(options) {
  return callJson(native.swiftImportersJson, options);
}

async function swiftTestTargets(options) {
  return callJson(native.swiftTestTargetsJson, options);
}

async function version() {
  return native.version();
}

module.exports = {
  analyzeProject,
  callSites,
  check,
  dataPw,
  deadExports,
  dependencies,
  dependents,
  effects,
  exportsOf,
  fetches,
  importers,
  infraOutputs,
  infraResourceRefs,
  infraTestFor,
  lockfileDiff,
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
  reactUsages,
  registryExtension,
  related,
  resolveCheck,
  rscCallers,
  serverRouteEdges,
  serverRouteList,
  serverRouteRelated,
  serverRoutes,
  swiftImporters,
  swiftTestTargets,
  symbols,
  testsComment,
  testsGraph,
  testsGraphMermaid,
  testsImpact,
  testsPlan,
  testsWhy,
  version,
};
