import { read } from '@example/db';
declare function opaque(value: unknown): void;
opaque(Object);
Object.freeze({ query: read }); // ADVISE_ESCAPED_OBJECT
