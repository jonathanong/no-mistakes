declare const extraOrigins: string[];
declare const runtimeCsp: string;
declare const runtimeSettings: { csp: string };
declare const baseMeta: Record<string, string>;
declare const computedKey: string;
declare function buildHeader(settings: { tenant: string }): string;
declare function originsFor(...directives: string[]): string[];
declare function buildCsp(...args: unknown[]): string;
declare function setHeader(name: string, value: string): void;

const { tenant } = { tenant: "acme" };
let uninitializedCsp: string;
void tenant;

export const CSP_HEADER = "Content-Security-Policy";
export const CSP_TEMPLATE = `Content-Security-Policy`;
export const CSP_ENABLED = true;
export const CSP_COUNT = 1;
export const CSP_NONE = null;
export const CSP_NEG = !CSP_ENABLED;
export function isCspEnabled(env: string) {
  return env === "prod";
}
export function emptyReturnCsp() {
  return;
}
export const buildStaticCsp = () => ["default-src 'self'", "connect-src 'self'"].join("; ");
export const aliasCsp = buildStaticCsp();
export const staticConstCsp = "default-src 'self'; connect-src 'self'";
export const aliasConstCsp = staticConstCsp;
export const staticJoinHeader = {
  name: "Content-Security-Policy",
  ...baseMeta,
  [computedKey]: 1,
  0: "ignored",
  value: ["default-src 'self'", "connect-src 'self'"].join("; "),
};
export const staticArrayHeader = {
  key: "Content-Security-Policy",
  value: ["default-src 'self'"],
};
export function applyStaticHeader() {
  setHeader("Content-Security-Policy", "default-src 'self'; connect-src 'self'");
}

export function buildOpaqueCsp(settings: { tenant: string }) {
  return buildHeader(settings);
}

export const buildArrowOpaqueCsp = (settings: { tenant: string }) => buildHeader(settings);

export const blockArrowCsp = (settings: { tenant: string }) => {
  const helper = () => 1;
  void helper;
  return buildHeader(settings);
};

export const fnExprCsp = function (settings: { tenant: string }) {
  return buildHeader(settings);
};

export const directOpaqueCsp = buildHeader({ tenant: "acme" });
export const opaqueIdentCsp = runtimeCsp;
export const memberCsp = runtimeSettings.csp;
export const _leadingUnderscoreCsp = buildHeader({ tenant: "acme" });
export const contentSecurityPolicy = buildHeader({ tenant: "acme" });

export const cspSourceMap = {
  ...baseMeta,
  "connect-src": ["'self'", ...extraOrigins],
};

export const spreadBuilderCsp = buildCsp(...["default-src 'self'"]);
export const spreadObjCsp = buildCsp({ ...{ "connect-src": extraOrigins } });
export const spreadArgInCall = buildCsp({ sources: [originsFor(...["connect-src"])] });

export const dynamicHeaderObj = {
  key: "Content-Security-Policy",
  value: buildHeader({ tenant: "acme" }),
};

export const dynamicHeaderMap = {
  "Content-Security-Policy": buildHeader({ tenant: "acme" }),
};

export function applyDynamicHeader(settings: { tenant: string }) {
  setHeader("Content-Security-Policy", buildHeader(settings));
}
