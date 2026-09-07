import { validateUrl, validateUrl as checkUrl } from "ssrf-guard/node";
import { "validateUrl" as quotedUrl } from "ssrf-guard/node";
import * as ssrf from "ssrf-guard/node";

const guard = require("ssrf-guard/node");
const { validateUrl: requireUrl } = require("ssrf-guard/node");
const typedGuard = require("ssrf-guard/node") as Guard;
const { validateUrl: fallback = noop } = require("ssrf-guard/node") as Guard;

export const { validateUrl: exportedUrl } = require("ssrf-guard/node");

export async function rejected(url: string, opts: object, key: string) {
  await checkUrl(url);
  await quotedUrl(url, {});
  await validateUrl(url, { ...opts });
  await ssrf.validateUrl(url, opts);
  await ssrf["validateUrl"](url, getOpts());
  await requireUrl(url, { other: 1 });
  await guard.validateUrl(url, { [key]: 1 });
  await typedGuard.validateUrl(url);
  await fallback(url, { ...opts });
  await exportedUrl(url);
  await require("ssrf-guard/node").validateUrl(url);
  await (require("ssrf-guard/node") as Guard).validateUrl(url, { ...opts });
}

export function laterRequire(url: string) {
  later(url);
}

const { validateUrl: later } = require("ssrf-guard/node");
