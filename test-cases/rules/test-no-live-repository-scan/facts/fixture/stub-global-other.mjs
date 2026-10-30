import { stubGlobal } from "vitest";

stubGlobal("fetch", () => {});
stubGlobal(`location`, {});
