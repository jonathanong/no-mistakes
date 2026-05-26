import { X } from "x";

const local = 1;

export { X as Y };
export { local as renamedLocal };
export { X as RemoteX } from "x";
