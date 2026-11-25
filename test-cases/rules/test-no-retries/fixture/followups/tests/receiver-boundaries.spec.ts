declare class ExternalSuite {
  runner: unknown;
}
declare const constructors: { Suite: typeof ExternalSuite };
const IndirectSuite = ExternalSuite;
new IndirectSuite().runner.describe.serial("local class alias", () => {});
new constructors.Suite().runner.describe.serial("dynamic constructor", () => {});

function unrelatedReceiver() {
  this.runner.describe.serial("ordinary function", () => {});
}
this.runner.describe.serial("module this", () => {});

class Registrations {
  add(suite: unknown) {
    suite.describe.serial("method parameter", () => {});
  }
}

function LegacySuite() {
  this.runner = makeUnrelatedRunner();
}
new LegacySuite().runner.describe.serial("unrelated constructor", () => {});
