const text = error.message;
// A with-object can replace either the assertion read or an initializer read.
with (scope) {
  expect(text).toBe("unknown");
  const inner = text;
  expect(inner).toBe("unknown");
}
