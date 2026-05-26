const testDirs: string[] = [];
const holder = {};

afterEach(() => {
  expect(testDirs).toHaveLength(1);
  holder.testDirs = true;
});

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});
