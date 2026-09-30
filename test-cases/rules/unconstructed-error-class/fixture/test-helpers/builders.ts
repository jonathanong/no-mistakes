import { HelperOnlyError } from "../src/helper-only";

export const fromHelper = new HelperOnlyError("only a shared test helper builds this");
