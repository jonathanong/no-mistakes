export function buildHeaderConfig(
  nonce: string,
  dynamicPart: string,
  sep: string,
  isDev: boolean,
  extra: string[],
  getParts: () => string[],
) {
  const resolved = [
    {
      key: "Content-Security-Policy",
      value:
        "default-src 'self'; script-src 'nonce-" +
        nonce +
        "' https://cdn.example.com; " +
        `img-src ${"https://img.example.com"}`,
    },
  ];
  const dynamicSep = ["default-src 'self'", "img-src https://img.example.com"].join(sep);
  const dynamicConcat = "default-src 'self'; " + dynamicPart + "; img-src https://img.example.com";
  const dynamicTemplate = `default-src 'self'; ${dynamicPart}; img-src https://img.example.com`;
  const badNonceSuffix = `script-src 'nonce-${nonce}'extra`;
  const dynamicBareArray = ["default-src 'self'", dynamicPart, "img-src https://img.example.com"];
  const filteredJoin = [
    "default-src 'self'",
    isDev && "script-src 'self' https://dev.example.com",
    "img-src https://img.example.com",
  ]
    .filter(Boolean)
    .join("; ");
  const logicalPolicy = isDev && "script-src 'self' https://dev.example.com";
  const ternaryJoin = isDev
    ? ["default-src 'self'", "script-src 'self'"].join("; ")
    : ["default-src 'self'"].join("; ");
  const nonArrayFiltered = extra.filter(Boolean).join("; ");
  const nonArrayCall = getParts().join("; ");
  return {
    resolved,
    dynamicSep,
    dynamicConcat,
    dynamicTemplate,
    badNonceSuffix,
    dynamicBareArray,
    filteredJoin,
    logicalPolicy,
    ternaryJoin,
    nonArrayFiltered,
    nonArrayCall,
  };
}
