import { beforeEach, describe, it } from "vitest";
import { WebClient } from "./client";
let uncalled: WebClient;
function never() {
  uncalled = new WebClient();
}
let conditional: WebClient;
if (false) {
  conditional = new WebClient();
}
let conditionalHook: WebClient;
false &&
  beforeEach(() => {
    conditionalHook = new WebClient();
  });
let ternaryHook: WebClient;
false
  ? beforeEach(() => {
      ternaryHook = new WebClient();
    })
  : null;
let loopHook: WebClient;
for (const _ of []) {
  beforeEach(() => {
    loopHook = new WebClient();
  });
}
let wrongHook: WebClient;
function arbitraryHook(callback: () => void) {}
arbitraryHook(() => {
  wrongHook = new WebClient();
});
let skippedHook: WebClient;
beforeEach.skip(() => {
  skippedHook = new WebClient();
});
let unknownTitle: WebClient;
declare const title: string;
describe(title, () => {
  beforeEach(() => {
    unknownTitle = new WebClient();
  });
});
let unrelated: WebClient;
describe("another test scope", () => {
  beforeEach(() => {
    unrelated = new WebClient();
  });
  it("another test", () => {});
});
let otherTest: WebClient;
it("initializes only itself", () => {
  otherTest = new WebClient();
});
describe("declaration scope", () => {
  let misplaced: WebClient;
});
describe("unrelated initialization scope", () => {
  beforeEach(() => {
    misplaced = new WebClient();
  });
  it("an unrelated declaration is not visible", () => {
    misplaced.request("/healthz");
  });
});
let duplicateTitle: WebClient;
describe("same title", () => {
  beforeEach(() => {
    duplicateTitle = new WebClient();
  });
});
describe("same title", () => {
  it("cannot inherit a sibling's hooks", () => {
    duplicateTitle.request("/healthz");
  });
});
beforeEach(() => {
  const hidden = new WebClient();
});
{
  const blockClient = new WebClient();
}
it("unexecuted initializers cannot establish helper ownership", () => {
  uncalled.request("/healthz");
  conditional.request("/healthz");
  conditionalHook.request("/healthz");
  ternaryHook.request("/healthz");
  loopHook.request("/healthz");
  wrongHook.request("/healthz");
  skippedHook.request("/healthz");
  unknownTitle.request("/healthz");
  unrelated.request("/healthz");
  otherTest.request("/healthz");
  hidden.request("/healthz");
  blockClient.request("/healthz");
});
