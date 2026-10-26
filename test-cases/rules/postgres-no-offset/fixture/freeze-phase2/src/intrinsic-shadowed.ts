import { read } from '@example/db';
declare function opaque(value: unknown): void;
function shadowed() {
  const Object = { freeze: opaque };
  Object.freeze({ query: read }); // ADVISE_SHADOWED_FREEZE
}
