const moduleSpecifier = "fixture-node-only/entry";
const moduleAlias = moduleSpecifier;
const { constantOnly } = await import(/* @vite-ignore */ moduleAlias);
constantOnly();

async function loadFunctionConstant() {
  const moduleSpecifier = "./targets/function-constant.mts";
  const loaded = await import(moduleSpecifier);
  loaded.functionConstantOnly();
}

async function loadUrlForms() {
  const { directOnly } = await import(new URL("./targets/url-direct.mts", import.meta.url));
  const specifier = new URL("./targets/url-href.mts", import.meta.url).href;
  const { hrefOnly } = await import(specifier);
  const { stringOnly } = await import(String(new URL("./targets/url-string.mts", import.meta.url)));
  const { pathnameOnly } = await import(new URL("./targets/url-pathname.mts", import.meta.url).pathname);
  directOnly(); hrefOnly(); stringOnly(); pathnameOnly();
}

export { loadFunctionConstant, loadUrlForms };
