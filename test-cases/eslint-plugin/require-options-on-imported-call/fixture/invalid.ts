import { validateUrl, validateUrl as checkUrl } from "ssrf-guard/node";
import { "validateUrl" as quotedUrl } from "ssrf-guard/node";
import * as ssrf from "ssrf-guard/node";

const guard = require("ssrf-guard/node");
const { validateUrl: namespacedUrl } = ssrf;
const { validateUrl: fromGuard } = guard;
const { validateUrl: requireUrl } = require("ssrf-guard/node");
const typedGuard = require("ssrf-guard/node") as Guard;
const { validateUrl: fallback = noop } = require("ssrf-guard/node") as Guard;
const staticMember = require(`ssrf-guard/node` as string)["validateUrl" as string];
const { [`validateUrl` as string]: typedDestructuredUrl } = require(`ssrf-guard/node` as string);
let mutableGuard = require(`ssrf-guard/node`);
var mutableUrl = require("ssrf-guard/node").validateUrl;

export const { validateUrl: exportedUrl } = require("ssrf-guard/node");

export async function rejected(url: string, opts: object, key: string) {
  await checkUrl(url);
  await checkUrl?.(url);
  await quotedUrl(url, {});
  await namespacedUrl(url);
  await fromGuard(url);
  await validateUrl(url, { ...opts });
  await ssrf.validateUrl(url, opts);
  await ssrf?.validateUrl(url);
  await ssrf["validateUrl"](url, getOpts());
  await requireUrl(url, { other: 1 });
  await guard.validateUrl(url, { [key]: 1 });
  await typedGuard.validateUrl(url);
  await fallback(url, { ...opts });
  await exportedUrl(url);
  await require("ssrf-guard/node").validateUrl(url);
  await (require("ssrf-guard/node") as Guard).validateUrl(url, { ...opts });
  await staticMember(url);
  await typedDestructuredUrl(url);
  await mutableGuard["validateUrl" as string](url);
  await mutableUrl(url);
}

export function laterRequire(url: string) {
  later(url);
}

const { validateUrl: later } = require("ssrf-guard/node");
