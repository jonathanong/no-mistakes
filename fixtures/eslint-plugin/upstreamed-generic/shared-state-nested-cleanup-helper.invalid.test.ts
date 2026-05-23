const testDirs: string[] = [];

afterEach(() => {
  function maybeCleanup() {
    testDirs.length = 0;
  }
});

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});
