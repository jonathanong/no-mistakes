const name = "dep";

export async function loadTemplate() {
  return import(`./${name}`);
}

export async function loadIdentifier(moduleName: string) {
  return import(moduleName);
}

export function loadRequire(moduleName: string) {
  return require(moduleName);
}

export function loadRequireResolve(moduleName: string) {
  return require.resolve(moduleName);
}

export function loadConcat(suffix: string) {
  return import("./" + suffix);
}

export const Lazy = dynamic(() => import("./dep"));

function dynamic(loader: () => Promise<unknown>) {
  return loader;
}
