const testDirs: string[] = [];

beforeAll(() => {
  testDirs.length = 0;
});

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});
