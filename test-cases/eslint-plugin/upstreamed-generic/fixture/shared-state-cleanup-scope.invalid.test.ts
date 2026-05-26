const testDirs: string[] = [];

describe("with cleanup", () => {
  afterEach(() => {
    testDirs.length = 0;
  });

  test("creates a temp dir", () => {
    testDirs.push("/tmp/example");
  });
});

describe("without cleanup", () => {
  test("creates a temp dir", () => {
    testDirs.push("/tmp/example");
  });
});
