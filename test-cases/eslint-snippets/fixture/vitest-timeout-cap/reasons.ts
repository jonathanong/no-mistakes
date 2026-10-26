import { test, it, describe, beforeAll, beforeEach, afterAll, afterEach, vi } from "vitest";
test("missing", () => {}, 6000);
// no-mistakes-timeout-reason: waits for the isolated worker to boot
test("explained", () => {}, 6000);
// no-mistakes-timeout-reason:
it("blank", () => {}, 6000);
// no-mistakes-timeout-reason: TODO explain worker startup
beforeAll(() => {}, 6000);
// no-mistakes-timeout-reason: TBD
beforeEach(() => {}, 6000);
// no-mistakes-timeout-reason: FIXME
 afterAll(() => {}, 6000);
// no-mistakes-timeout-reason: XXX
afterEach(() => {}, 6000);
// no-mistakes-timeout-reason: placeholder
describe("suite", () => {}, 6000);
test("property", {
  // no-mistakes-timeout-reason: fixture boots an isolated worker
  timeout: 6000,
}, () => {});
test("value", { timeout: /* no-mistakes-timeout-reason: fixture boots an isolated worker */ 6000 }, () => {});
// no-mistakes-timeout-reason: bootstraps a shared worker
vi.setConfig({ testTimeout: 6000, hookTimeout: 6000 });
vi.setConfig({ testTimeout: 6000, hookTimeout: 6000 });
const extended = test.extend({});
extended("extended", () => {}, 6000);
const alias = it;
alias("alias", () => {}, 6000);
// no-mistakes-timeout-reason: allowed suite setup
describe("outer", { timeout: 6000 }, () => {
  test("nested missing", () => {}, 6000);
});
// no-mistakes-timeout-reason: unrelated statement
const unrelated = 1;
test("unrelated", () => {}, 6000);
const prior = 1; // no-mistakes-timeout-reason: trailing prior statement
test("prior", () => {}, 6000);
// no-mistakes-timeout-reason: separated

test("separated", () => {}, 6000);
// no-mistakes-timeout-reason: priority retains hard cap
test("too large", () => {}, 30001);
// no-mistakes-timeout-reason: priority retains positivity
test("disabled", () => {}, 0);
test("boundary", () => {}, 5000);
// no-mistakes-timeout-reason: known only at runtime
test("unknown", () => {}, process.env.TIMEOUT);
const shared = {
  // no-mistakes-timeout-reason: shared fixture boots an isolated worker
  timeout: 6000,
};
test("shared property", shared, () => {});
// no-mistakes-timeout-reason: scalar declarations do not justify their consumers
const timeout = 6000;
test("scalar use", () => {}, timeout);
// no-mistakes-timeout-reason: starts an isolated worker before the suite
beforeAll(() => {}, 6000);
// no-mistakes-timeout-reason: resets the isolated worker before each case
beforeEach(() => {}, 6000);
// no-mistakes-timeout-reason: stops the isolated worker after the suite
afterAll(() => {}, 6000);
// no-mistakes-timeout-reason: resets the isolated worker after each case
afterEach(() => {}, 6000);
// no-mistakes-timeout-reason: waits for the extended worker fixture
extended("explained extended", () => {}, 6000);
// no-mistakes-timeout-reason: waits for the isolated worker fixture
alias("explained local alias", () => {}, 6000);
import { it as renamed } from "vitest";
renamed("missing renamed native alias", () => {}, 6000);
// no-mistakes-timeout-reason: waits for the isolated worker fixture
renamed("explained renamed native alias", () => {}, 6000);
// no-mistakes-disable-next-line vitest-timeout-cap -- this fixture verifies reason suppression
test("suppressed reason", () => {}, 6000);
