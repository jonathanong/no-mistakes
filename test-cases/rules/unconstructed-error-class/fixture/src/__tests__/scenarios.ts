import { UnusedError } from "../errors";
import { Lib } from "../namespace-lib";

export const fromTestsDirectory = new UnusedError("a __tests__ file does not count");
export const namespaceMemberFromTests = new Lib.TestOnlyBuilt();
