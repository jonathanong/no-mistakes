console.log("browser-suite-executed");
process.exitCode = Number(process.env.BROWSER_FIXTURE_EXIT_CODE || 0);
