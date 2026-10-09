function sloppyReplacement(statement) {
  const ignored = (arguments[0] = "replacement");
  return statement;
}

function deleteAndRecreate(statement) {
  delete arguments[0];
  const ignored = (arguments[0] = "replacement");
  return statement;
}

module.exports = sloppyReplacement;
