// The job's return type reaches a barrel containing independent utility aliases.
// Loading that barrel must not turn erased type syntax into an unproven callback.
export type * from "./representation";
export type * from "./pair";
export type * from "./result";
