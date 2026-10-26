export function shadowed(registry: any) {
  const Intl = registry;
  const Map = registry;
  new Intl.DateTimeFormat();
  new Map();
}
import { UsedError as Map } from '@app/errors';
new Map();
declare const opaque: any;
export namespace NamespaceGlobals {
  const Intl = opaque;
  const WebAssembly = opaque;
  export function format() { return new Intl.NumberFormat(); }
  export function compile() { return new WebAssembly.Module(); }
}
