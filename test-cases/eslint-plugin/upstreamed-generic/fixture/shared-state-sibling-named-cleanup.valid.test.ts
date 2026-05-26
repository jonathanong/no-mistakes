const firstDirs: string[] = [];
const secondDirs: string[] = [];

describe("first", () => {
  afterEach(cleanup);

  function cleanup() {
    firstDirs.length = 0;
  }

  test("creates a temp dir", () => {
    firstDirs.push("/tmp/first");
  });
});

describe("second", () => {
  afterEach(cleanup);

  function cleanup() {
    secondDirs.length = 0;
  }

  test("creates a temp dir", () => {
    secondDirs.push("/tmp/second");
  });
});
