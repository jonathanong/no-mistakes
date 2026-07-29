const fixtureRoots = ["fixtures/", "test-cases/"];

function isFixturePath(path) {
  return fixtureRoots.some((root) => path.startsWith(root));
}

function isFixtureOnlyChange(paths) {
  return paths.length > 0 && paths.every(isFixturePath);
}

function shouldCloseFixtureOnlyChange(paths, changedFileCount) {
  return paths.length === changedFileCount && isFixtureOnlyChange(paths);
}

module.exports = {
  isFixtureOnlyChange,
  shouldCloseFixtureOnlyChange,
};
