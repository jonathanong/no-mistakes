import { read } from '@example/db';
declare function opaque(value: unknown): void;
globalThis.Object = { freeze: opaque } as ObjectConstructor;
Object.freeze({ query: read }); // ADVISE_GLOBALTHIS_MUTATION
