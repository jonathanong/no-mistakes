const testDirs: string[] = [];

afterEach(cleanupDirs);

function cleanupDirs() {
  testDirs.length = 0;
}

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});
