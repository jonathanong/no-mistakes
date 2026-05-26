const TEST_TIMEOUT = 1000;
const testDirs: string[] = [];

describe("suite", () => {
  afterEach(cleanupDirs, TEST_TIMEOUT);

  function cleanupDirs() {
    testDirs.length = 0;
  }

  test("creates a temp dir", () => {
    testDirs.push("/tmp/example");
  });
});
