const kind = "users";
const args = ["./users.test"];
require(`./${kind}.test`);
require(1);
require.apply(null, args);
