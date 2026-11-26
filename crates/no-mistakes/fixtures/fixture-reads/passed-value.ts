consume(loadFixture("passed-value").label);
const loaded = ["passed-value"].map(loadFixture);
consume(loaded);
