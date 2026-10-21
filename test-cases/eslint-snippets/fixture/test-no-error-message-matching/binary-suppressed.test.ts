declare const err: { message: string };

test("binary", () => {
  // no-mistakes-disable-next-line test-no-error-message-matching -- retained probe
  if (err.message === "x") return;
  if (err.message === "y") return;
});
