"use strict";

const native = require("./bin/no-mistakes.node");
const planning = require("./planning");

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

async function importUsages(options) {
  return callJson(native.importUsagesJson, options);
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

async function ciImpact(options) {
  return callJson(native.ciImpactJson, options);
}

async function ciEnv(options) {
  return callJson(native.ciEnvJson, options);
}

async function impactedChecks(options) {
  return callJson(native.impactedChecksJson, options);
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
  ciEnv,
  ciImpact,
  dataPw,
  deadExports,
  dependencies,
  dependents,
  effects,
  exportsOf,
  fetches,
  impactedChecks,
  importUsages,
  importers,
  infraOutputs,
  infraResourceRefs,
  infraTestFor,
  lockfileDiff,
  playwrightCheck,
  playwrightEdges,
  playwrightRelated,
  playwrightTests,
  reactAnalyze,
  reactCheck,
  reactUsages,
  registryExtension,
  related,
  resolveCheck,
  rscCallers,
  swiftImporters,
  swiftTestTargets,
  symbols,
  version,
  ...planning,
};
