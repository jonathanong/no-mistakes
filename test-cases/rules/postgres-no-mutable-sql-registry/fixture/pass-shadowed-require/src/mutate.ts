function require(path) { return { tables: {} }; }
const registry = require("./tables");
registry.tables.users = "app.users";
