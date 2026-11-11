import { it, "test" as scenario } from "vitest";
it("spread arguments", ...options);
it("primitive options", undefined, () => {});
it("numeric timeout", () => {}, 100);
it("known other option", { timeout: 100 }, () => {});
it("spread options", { ...options }, () => {});
it("computed options", { [key]: true }, () => {});
scenario("quoted runner", { concurrent: true }, () => {});
