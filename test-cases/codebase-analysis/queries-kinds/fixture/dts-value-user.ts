// A value import of a declaration-only module does not resolve at runtime.
import { Foo } from "./types";

export const f: Foo = { x: 1 };
