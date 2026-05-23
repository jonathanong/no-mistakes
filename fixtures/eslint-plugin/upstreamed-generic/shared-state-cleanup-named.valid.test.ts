let testDirs: string[] = [];

afterEach(cleanupDirs);

function cleanupDirs() {
  testDirs = [];
}

test("creates a temp dir", () => {
  testDirs.push("/tmp/example");
});
