"use strict";

function readTimeoutOptions(option) {
  const max = option.max ?? 30000;
  const fixtureMax = option.fixtureMax ?? max;
  for (const [key, value] of [
    ["max", max],
    ["fixtureMax", fixtureMax],
  ])
    if (!Number.isFinite(value) || value <= 0)
      throw new Error(`playwright-test-timeout-cap ${key} must be positive and finite`);
  for (const [source, roles] of Object.entries(option.exportRoles ?? {})) {
    if (source === "@playwright/test" && Object.keys(roles).length)
      throw new Error(
        "playwright-test-timeout-cap exportRoles cannot override builtin SDK exports",
      );
    if (["test", "defineConfig", "mergeTests"].some((key) => Object.hasOwn(roles, key)))
      throw new Error(
        "playwright-test-timeout-cap exportRoles cannot override recognized registrar exports",
      );
  }
  return { max, fixtureMax };
}

module.exports = { readTimeoutOptions };
