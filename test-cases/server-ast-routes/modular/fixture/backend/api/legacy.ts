import express = require("express");

const legacy = express.Router();

legacy.get("/:id", getLegacy);

export { legacy };

function getLegacy() {}
