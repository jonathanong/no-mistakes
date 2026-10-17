"use strict";

const fs = require("node:fs/promises");

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

module.exports = { camelizeValue, decamelizePlanOptions, loadPlanJson, readPlanFile };
