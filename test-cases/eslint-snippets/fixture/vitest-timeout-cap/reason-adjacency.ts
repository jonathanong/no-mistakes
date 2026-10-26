// no-mistakes-timeout-reason: waits for a worker to boot
test("global reason", () => {}, 6000);
// no-mistakes-timeout-reason: explanation separated by another comment

// no-mistakes-timeout-reason: TODO worker startup
test("blank line between comments", () => {}, 6000);
// no-mistakes-timeout-reason: worker startup
(test)("parenthesized registration", () => {}, 6000);
// no-mistakes-timeout-reason: worker startup
test("wrapped timeout", () => {}, (6000));
test("wrapped value annotation", () => {}, /* no-mistakes-timeout-reason: worker startup */ (6000));
test("nested grouped value", () => {}, /* no-mistakes-timeout-reason: worker startup */ (((6000))));
test("ordinary comment before grouped value", () => {}, /* no-mistakes-timeout-reason: worker startup */ (
  // Keep the grouping comment between the marker and literal.
  (6000)
));
test("blank before grouped value", () => {}, /* no-mistakes-timeout-reason: worker startup */ (

  6000
));
test("intervening callback", /* no-mistakes-timeout-reason: worker startup */ () => {}, (6000));
const prior = 1; /* no-mistakes-timeout-reason: worker startup */
test("prior statement reason", () => {}, (6000));
// A reason for the outer call must not justify its inner registration.
/* no-mistakes-timeout-reason: worker startup */
consume(test("wrong outer reason", () => {}, (6000)));
// The outer call's required parentheses must not become grouping for the test.
/* no-mistakes-timeout-reason: worker startup */
consume(
  test("wrong outer delimiter", () => {}, (6000))
);
test("grouping over several lines", () => {}, /* no-mistakes-timeout-reason: worker startup */ (
  (
    (6000)
  )
));
// Generic adjacency must distinguish syntax parentheses from grouping.
if /* adjacency: mandatory condition */ (6000) {}
do {} while /* adjacency: mandatory condition */ (6000);
consume /* adjacency: mandatory argument */ (6000);
new Consumer /* adjacency: mandatory argument */ (6000);
const grouped = (
  /* adjacency: grouped value */ ((6000))
);
consume?.
// no-mistakes-timeout-reason: worker startup
(test("wrong optional outer delimiter", () => {}, 6000));
consume
// no-mistakes-timeout-reason: worker startup
(test?.("wrong chained inner registration", () => {}, 6000));
(consume)
// no-mistakes-timeout-reason: worker startup
(test("wrong grouped callee delimiter", () => {}, 6000));
