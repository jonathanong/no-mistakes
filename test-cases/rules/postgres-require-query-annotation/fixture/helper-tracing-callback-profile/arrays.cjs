const database = require("@app/db");
function createArrays() {
  const shared = () => database.query(statement);
  const child = [shared];
  const alias = child;
  const nested = [alias, child];
  // Shared children and a self-cycle must neither duplicate nor loop traversal.
  nested[2] = nested;
  const removed = [() => database.query("SELECT deleted")];
  const recreated = [() => database.query("SELECT replaced")];
  recreated[0] = shared;
  const installer = [() => {
    installer[1] = () => database.query("SELECT installed");
  }];
  const statement = "SELECT 1";
  return [nested, removed, recreated, installer];
}
