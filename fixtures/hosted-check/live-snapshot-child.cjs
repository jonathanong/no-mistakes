// A bounded healthy run and a deliberately stalled run share a saved process.
if (process.argv.includes("--hang")) {
  process.stdout.write("fixture analyzer started\n");
  setInterval(() => {}, 1000);
} else if (process.argv.includes("--fast")) setTimeout(() => {}, 10);
else setTimeout(() => {}, 400);
