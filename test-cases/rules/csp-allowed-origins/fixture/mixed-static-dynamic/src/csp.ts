declare const thirdPartyOrigins: string[];
declare const uploadOrigins: string[];
declare function originsFor(directive: "connect-src"): string[];
declare function buildCsp(config: { sources: Record<string, string[]> }): string;

export const buildStaticCsp = () => ["default-src 'self'", "connect-src 'self'"].join("; ");

export function buildAppCsp(settings: { tenant: string }) {
  void settings;
  return buildCsp({
    sources: {
      "connect-src": [...thirdPartyOrigins, ...originsFor("connect-src"), ...uploadOrigins],
    },
  });
}
