import { test as base } from "@playwright/test";
import { base as localBase } from "./local-base";

export function createMixedFactory() {
  return process.env.CI ? base : Object.assign({}, base);
}

export function createControlFlowFactory() {
  for (const candidate of [base]) {
    return candidate;
  }
  return base;
}

export function createLocalUncertainFactory() {
  return process.env.CI ? localBase : Object.assign({}, localBase);
}

export function createResolvedFactory() {
  return base.extend({});
}

export const createArrowResolved = () => base.extend({});

export const createArrowUnresolved = () => Object.assign({}, base);

export function createUnrelatedFactory() {
  return { describe: { serial() {}, configure() {} } };
}
