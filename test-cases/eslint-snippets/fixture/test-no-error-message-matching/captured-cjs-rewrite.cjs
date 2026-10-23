const error = { message: "captured", code: "CAPTURED" };
const code = error.code;
const safe = () => {
  expect(code).toBe("CAPTURED");
};
safe();
const optional = () => {
  // Optional eval is indirect and cannot insert a nearer local code binding.
  eval?.("var code = 'ordinary'");
  expect(code).toBe("CAPTURED");
};
const referenceOnly = () => {
  const indirect = eval;
  indirect("var code = 'ordinary'");
  expect(code).toBe("CAPTURED");
};
optional();
referenceOnly();
