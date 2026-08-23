import { value } from "../src/value";

// The configured Vitest wrapper reaches this test through a package-local alias.
test("catalog-scoped integration", /* no-mistakes: integration=openai */ () => {
  void value;
});
