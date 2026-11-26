class Suite {
  runner = this.runner;
}

new Suite().runner.describe.serial("ordered", () => {});
