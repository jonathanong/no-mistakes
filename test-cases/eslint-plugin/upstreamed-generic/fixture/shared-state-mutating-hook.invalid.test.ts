const testDirs: string[] = [];

afterEach(() => {
  testDirs.push("/tmp/marker");
});

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});
