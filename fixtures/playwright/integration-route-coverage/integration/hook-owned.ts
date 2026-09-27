import { beforeEach, beforeAll, describe, it } from "vitest";
import { WebClient } from "./client";

let client: WebClient;
let allClient: WebClient;
describe("real registered integration hooks", function () {
  beforeEach(() => {
    client = new WebClient();
  });
  beforeAll(function () {
    allClient = new WebClient();
  });
  describe("nested consumer tests inherit ancestor hooks", () => {
    it("direct callbacks use their hook-initialized clients", () => {
      client.request("/hook-ready");
      client.loadPage("/hook-page");
      allClient.request("/hook-all");
    });
    it("a per-test constructor belongs only to its own callback", () => {
      const local = new WebClient();
      local.request("/local-ready");
    });
  });
});
