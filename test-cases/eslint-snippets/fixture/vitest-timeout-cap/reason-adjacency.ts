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
