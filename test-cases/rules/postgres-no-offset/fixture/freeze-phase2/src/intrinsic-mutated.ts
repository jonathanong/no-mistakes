import { read } from '@example/db';
declare function opaque(value: unknown): void;
Object.freeze = opaque;
Object.freeze({ query: read }); // ADVISE_MUTATED_FREEZE
