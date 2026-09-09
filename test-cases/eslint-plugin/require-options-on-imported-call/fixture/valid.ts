import validateUrlDefault, { validateUrl, validateUrl as checkUrl } from "ssrf-guard/node";
import { "validateUrl" as quotedUrl } from "ssrf-guard/node";
import * as ssrf from "ssrf-guard/node";
import * as other from "other-guard/node";

const guard = require("ssrf-guard/node");
const { validateUrl: namespacedUrl } = ssrf;
const { validateUrl: fromGuard } = guard;
const { validateUrl: requireUrl } = require("ssrf-guard/node");
const typedGuard = require("ssrf-guard/node") as Guard;
const { ...rest } = require("ssrf-guard/node");
const { [`validateUrl` as string]: typedDestructuredUrl } = require(`ssrf-guard/node` as string);
let mutableGuard = require(`ssrf-guard/node`);
var mutableUrl = require("ssrf-guard/node").validateUrl;

export const { validateUrl: exportedUrl } = require("ssrf-guard/node");

export async function accepted(
  url: string,
  signal: AbortSignal,
  timeoutMs: number,
  rest: object,
  opts: object,
) {
  await checkUrl(url, { timeoutMs });
  await checkUrl?.(url, { timeoutMs });
  await quotedUrl(url, { signal });
  await namespacedUrl(url, { timeoutMs });
  await fromGuard(url, { signal });
  await ssrf.validateUrl(url, { signal });
  await ssrf?.validateUrl(url, { signal });
  await ssrf["validateUrl"](url, { timeoutMs: DNS_TIMEOUT_MS });
  await requireUrl(url, { timeoutMs, signal });
  await guard.validateUrl(url, { timeoutMs: timeoutMs, ...rest });
  await typedGuard.validateUrl(url, { ...opts, signal } as Options);
  await validateUrl(url, { timeoutMs } satisfies Options);
  await validateUrl(url, { ["timeoutMs"]: 1 });
  await exportedUrl(url, { signal: signal });
  await require("ssrf-guard/node").validateUrl(url, { timeoutMs: 1 });
  await (require("ssrf-guard/node") as Guard).validateUrl(url, { timeoutMs });
  await typedDestructuredUrl(url, { ["timeoutMs" as string]: 1 });
  await mutableGuard["validateUrl" as string](url, { timeoutMs });
  await mutableUrl(url, { signal });
}

export async function ignored(
  url: string,
  validateUrl: (url: string) => void,
  deps: { validateUrl: (url: string) => void },
  dynamic: string,
) {
  validateUrl(url);
  validateUrlDefault(url);
  other.validateUrl(url);
  ssrf.otherFn(url);
  deps.validateUrl(url);
  const alias = checkUrl;
  alias(url);
  ssrf[dynamic](url);
  rest.validateUrl(url);
  const { [dynamic]: skipped } = ssrf;
  skipped(url);
  const { ...nsRest } = ssrf;
  nsRest.validateUrl(url);
}

export async function laterRequire(url: string) {
  await later(url, { timeoutMs: 1 });
}

const { validateUrl: later } = require("ssrf-guard/node");
