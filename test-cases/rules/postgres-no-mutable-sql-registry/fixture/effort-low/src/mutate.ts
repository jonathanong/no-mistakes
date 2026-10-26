import { tables } from "./tables";
// A const assertion is erased; this assignment still mutates the runtime object.
// @ts-expect-error Deliberately demonstrate that a readonly type is not a freeze.
tables.users = "other.users";
