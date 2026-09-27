import { it } from "vitest";
import { WebClient } from "./client";
import { WebClient as Lookalike } from "./lookalike";

let replaced = new WebClient();
replaced = new Lookalike();
let destructured = new WebClient();
[destructured] = [new Lookalike()];
const untouched = new WebClient();
const nestedMember = new WebClient();
[nestedMember.request] = [async () => {}];
const computedMember = new WebClient();
declare const method: string;
({ result: computedMember[method] } = { result: async () => {} });
let defaulted = new WebClient();
[defaulted = new WebClient()] = [];
let shorthand = new WebClient();
({ shorthand = new WebClient() } = {});
// Mutating an unrelated temporary instance must not invalidate this receiver.
new WebClient().request = async () => {};

it("only the untouched receiver remains owned by the helper", () => {
  (() => {})();
  replaced.request("/healthz");
  destructured.request("/healthz");
  nestedMember.request("/healthz");
  computedMember.request("/healthz");
  defaulted.request("/healthz");
  shorthand.request("/healthz");
  untouched.request("/other");
});
