import { Aliased } from '@app/plain';
export function dynamic(flag: boolean, registry: any) {
  new (flag ? Aliased : registry)();
  let Constructor = Aliased;
  if (flag) Constructor = registry;
  new Constructor();
  const WebAssembly = registry;
  new WebAssembly.Module();
  new Intl.MissingFormat();
  new globalThis.WebAssembly.RuntimeError();
}
