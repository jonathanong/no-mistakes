#!/usr/bin/env node
"use strict";

const { run } = require("./cli");

function main() {
  process.exit(run());
}

if (require.main === module) main();

module.exports = { main };
