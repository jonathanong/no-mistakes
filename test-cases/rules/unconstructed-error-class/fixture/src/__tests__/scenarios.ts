import { UnusedError } from "../errors";

export const fromTestsDirectory = new UnusedError("a __tests__ file does not count");
