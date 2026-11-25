declare const extraOrigins: string[];
declare function buildHeader(settings: { tenant: string }): string;
declare function setHeader(name: string, value: string): void;

export const CSP_HEADER = "Content-Security-Policy";
export const CSP_ENABLED = true;
export function isCspEnabled(env: string) {
  return env === "prod";
}
export const buildStaticCsp = () => ["default-src 'self'", "connect-src 'self'"].join("; ");
export const aliasCsp = buildStaticCsp();
export const staticConstCsp = "default-src 'self'; connect-src 'self'";
export const aliasConstCsp = staticConstCsp;
export function applyStaticHeader() {
  setHeader("Content-Security-Policy", "default-src 'self'; connect-src 'self'");
}

export function buildOpaqueCsp(settings: { tenant: string }) {
  return buildHeader(settings);
}

export const buildArrowOpaqueCsp = (settings: { tenant: string }) => buildHeader(settings);

export const directOpaqueCsp = buildHeader({ tenant: "acme" });

export const cspSourceMap = {
  "connect-src": ["'self'", ...extraOrigins],
};

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
