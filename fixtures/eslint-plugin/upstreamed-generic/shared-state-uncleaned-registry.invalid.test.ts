const testDirs: string[] = [];
const holder = {};

afterEach(() => {
  holder.testDirs = true;
});

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});
