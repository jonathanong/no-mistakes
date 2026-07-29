const fixtureRoots = ["fixtures/", "test-cases/"];

function isFixturePath(path) {
  return fixtureRoots.some((root) => path.startsWith(root));
}

function isFixtureOnlyChange(paths) {
  return paths.length > 0 && paths.every(isFixturePath);
}

function fixtureOnlyDecision(paths, changedFileCount) {
  if (paths.length !== changedFileCount) {
    return "incomplete";
  }

  return isFixtureOnlyChange(paths) ? "close" : "continue";
}

module.exports = {
  fixtureOnlyDecision,
  isFixtureOnlyChange,
};
