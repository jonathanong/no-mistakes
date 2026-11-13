#!/usr/bin/env node
console.log(JSON.stringify({ args: process.argv.slice(2), inherited: process.env.NEXTEST_RUNNER_FIXTURE_VALUE }));
console.error("fixture stderr preserved");
if (process.env.NEXTEST_RUNNER_FIXTURE_SIGNAL) process.kill(process.pid, process.env.NEXTEST_RUNNER_FIXTURE_SIGNAL);
else process.exitCode = Number(process.env.NEXTEST_RUNNER_FIXTURE_STATUS || 0);
