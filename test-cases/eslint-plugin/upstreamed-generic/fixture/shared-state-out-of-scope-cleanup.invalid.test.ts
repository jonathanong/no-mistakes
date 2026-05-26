const testDirs: string[] = [];

afterEach(cleanup);

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});

test("declares unrelated cleanup", () => {
  function cleanup() {
    testDirs.length = 0;
  }
});
