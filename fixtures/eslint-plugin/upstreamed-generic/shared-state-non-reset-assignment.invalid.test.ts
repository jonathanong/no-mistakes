let testDirs: string[] = [];

afterEach(() => {
  testDirs = testDirs.concat("/tmp/marker");
});

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});
