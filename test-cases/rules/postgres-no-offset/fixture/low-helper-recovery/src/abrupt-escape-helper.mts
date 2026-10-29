import { unknownEscape } from './missing-source.mjs';
// This source-visible helper has no selected SQL; the inner call has no source body.
export function escaping(value: string): never {
  throw unknownEscape(value);
}
