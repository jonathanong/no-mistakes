const { readFileSync } = require("node:fs");

const DIMENSIONS = ["lines", "functions", "regions"];

function parseFloor(value, dimension) {
  if (typeof value !== "string" || !/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?$/i.test(value)) {
    throw new Error(`${dimension} floor must be a finite percentage between 0 and 100`);
  }
  const floor = Number(value);
  if (!Number.isFinite(floor) || floor < 0 || floor > 100) {
    throw new Error(`${dimension} floor must be a finite percentage between 0 and 100`);
  }
  return floor;
}

function evaluateCoverage(summary, floors) {
  if (!Array.isArray(summary?.data) || summary.data.length === 0) {
    throw new Error("coverage summary must contain a nonempty data array");
  }
  return DIMENSIONS.map((dimension) => {
    const floor = parseFloor(floors[dimension], dimension);
    let count = 0;
    let covered = 0;
    for (const [index, data] of summary.data.entries()) {
      const total = data?.totals?.[dimension];
      if (
        !Number.isSafeInteger(total?.count) ||
        !Number.isSafeInteger(total?.covered) ||
        total.count < 0 ||
        total.covered < 0 ||
        total.covered > total.count
      ) {
        throw new Error(
          `data[${index}].totals.${dimension} requires valid count and covered integers`,
        );
      }
      count += total.count;
      covered += total.covered;
      if (!Number.isSafeInteger(count) || !Number.isSafeInteger(covered)) {
        throw new Error(`${dimension} totals exceed exact integer precision`);
      }
    }
    // Match cargo-llvm-cov get_coverage_percent: aggregate counts first, then
    // compare the unrounded ratio. Exported "percent" values are only display data.
    const percent = count === 0 ? 0 : (covered * 100) / count;
    return { dimension, count, covered, percent, floor, passed: percent >= floor };
  });
}

function main(args) {
  if (args.length !== 4) {
    throw new Error(
      "Usage: rust-coverage-floors.cjs SUMMARY_JSON LINE_FLOOR FUNCTION_FLOOR REGION_FLOOR",
    );
  }
  const [path, ...values] = args;
  const floors = Object.fromEntries(
    DIMENSIONS.map((dimension, index) => [dimension, values[index]]),
  );
  const results = evaluateCoverage(JSON.parse(readFileSync(path, "utf8")), floors);
  for (const { dimension, count, covered, percent, floor, passed } of results) {
    console.log(
      `${dimension}: ${percent}% (${covered}/${count}); minimum ${floor}%${passed ? "" : " FAIL"}`,
    );
  }
  return results.every(({ passed }) => passed) ? 0 : 1;
}

if (require.main === module) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(`Rust coverage floor check failed: ${error.message}`);
    process.exitCode = 1;
  }
}

module.exports = { evaluateCoverage, parseFloor };
