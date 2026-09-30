import { TestOnlyError } from "./errors";

export const fromTest = new TestOnlyError("only a test builds this");
