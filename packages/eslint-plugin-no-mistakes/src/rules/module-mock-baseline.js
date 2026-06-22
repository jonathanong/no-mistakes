"use strict";

const { repoRelativeFilename } = require("./module-mock-helpers");

function baselineKey(filename, specifier) {
  return JSON.stringify([repoRelativeFilename(filename), specifier]);
}

function baselineSet(entries = []) {
  return new Set(entries.map(([file, specifier]) => JSON.stringify([file, specifier])));
}

function baselineMap(entries = []) {
  return new Map(
    entries.map(([file, specifier, count]) => [JSON.stringify([file, specifier]), count]),
  );
}

module.exports = {
  baselineKey,
  baselineMap,
  baselineSet,
};
