const error = { message: "captured" };
const text = error.message;
const check = () => {
  // TypeScript assertions do not turn a direct eval call into an indirect one.
  (eval as typeof eval)("var text = 'ordinary'");
  expect(text).toBe("ordinary");
};
check();
const nestedBlock = () => {
  {
    (eval as typeof eval)("var text = 'ordinary'");
  }
  expect(text).toBe("ordinary");
};
nestedBlock();
