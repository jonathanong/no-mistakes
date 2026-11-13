#!/usr/bin/env node
const fs = require("node:fs");
fs.appendFileSync(process.env.NATIVE_FIXTURE_EXECUTIONS, `${JSON.stringify({ executable: process.argv[1], args: process.argv.slice(2), cwd: process.cwd() })}\n`);
process.exit(Number(process.env.NATIVE_TEST_FIXTURE_EXIT || 0));
