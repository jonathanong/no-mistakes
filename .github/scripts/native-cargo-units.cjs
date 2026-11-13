const path = require("node:path");

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, canonical(value[key])]),
    );
  return value;
}

function strings(value) {
  return Array.isArray(value) && value.every((entry) => typeof entry === "string");
}

function logicalFilename(filename, root, targetDirectory) {
  const paths = /^[A-Za-z]:[\\/]/.test(root) ? path.win32 : path;
  const absolute = paths.resolve(root, filename);
  const target = paths.relative(paths.resolve(root, targetDirectory), absolute);
  if (target !== ".." && !target.startsWith(`..${paths.sep}`) && !paths.isAbsolute(target))
    return `target/${target.split(paths.sep).join("/")}`;
  return paths.relative(root, absolute).split(paths.sep).join("/");
}

function counts(units) {
  return {
    fresh: units.filter((unit) => unit.fresh).length,
    compiled: units.filter((unit) => !unit.fresh).length,
  };
}

function collectCargoUnits(output, { root, targetDirectory = "target", context = {} }) {
  if (typeof root !== "string" || !root) throw new Error("Cargo unit collection requires root");
  const units = [];
  for (const [index, line] of String(output).split(/\r?\n/).entries()) {
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      if (/"reason"\s*:\s*"compiler-artifact"/.test(line))
        throw new Error(`Malformed Cargo compiler-artifact JSON at line ${index + 1}`);
      continue;
    }
    if (message?.reason !== "compiler-artifact") continue;
    if (
      typeof message.package_id !== "string" ||
      !message.package_id ||
      typeof message.target?.name !== "string" ||
      !message.target.name ||
      !strings(message.target.kind) ||
      !strings(message.target.crate_types) ||
      !strings(message.features) ||
      !strings(message.filenames) ||
      !message.profile ||
      typeof message.profile !== "object" ||
      Array.isArray(message.profile) ||
      typeof message.profile.test !== "boolean" ||
      typeof message.fresh !== "boolean"
    )
      throw new Error(`Incomplete Cargo compiler-artifact at line ${index + 1}`);
    units.push({
      package_id: message.package_id,
      target: {
        name: message.target.name,
        kind: [...message.target.kind].sort(),
        crate_types: [...message.target.crate_types].sort(),
      },
      features: [...message.features].sort(),
      profile: canonical(message.profile),
      fresh: message.fresh,
      filenames: message.filenames
        .map((filename) => logicalFilename(filename, root, targetDirectory))
        .sort(),
    });
  }
  units.sort((a, b) => {
    const left = JSON.stringify(a);
    const right = JSON.stringify(b);
    return left < right ? -1 : left > right ? 1 : 0;
  });
  return { schemaVersion: 1, context: canonical(context), counts: counts(units), units };
}

function signature(unit) {
  return canonical({
    package_id: unit.package_id,
    target: unit.target,
    features: unit.features,
    profile: unit.profile,
  });
}

function rootUnits(report, { rootPackageId, rootPackageName = "no-mistakes" }) {
  return report.units.filter((unit) => {
    if (rootPackageId) return unit.package_id === rootPackageId;
    const fragment = unit.package_id.split("#").at(-1);
    return (
      fragment.split("@")[0] === rootPackageName ||
      unit.package_id.startsWith(`${rootPackageName} `) ||
      unit.package_id.split("#")[0].split("/").at(-1) === rootPackageName
    );
  });
}

function compareUnits(previous, next) {
  function index(units) {
    const map = new Map();
    for (const unit of units) {
      const key = JSON.stringify(signature(unit));
      if (!map.has(key)) map.set(key, []);
      map.get(key).push(unit);
    }
    return map;
  }
  const left = index(previous);
  const right = index(next);
  const matching = [];
  const onlyBefore = [];
  const onlyAfter = [];
  for (const key of [...new Set([...left.keys(), ...right.keys()])].sort()) {
    const entry = { signature: JSON.parse(key) };
    if (left.has(key)) entry.before = counts(left.get(key));
    if (right.has(key)) entry.after = counts(right.get(key));
    if (left.has(key) && right.has(key)) matching.push(entry);
    else if (left.has(key)) onlyBefore.push(entry);
    else onlyAfter.push(entry);
  }
  return { matching, onlyBefore, onlyAfter };
}

function compareCargoUnits(before, after, options = {}) {
  const previous = rootUnits(before, options);
  const next = rootUnits(after, options);
  return {
    schemaVersion: 1,
    evidence:
      "Matching reported unit signatures do not establish reuse. Cargo fresh=true is the reuse evidence for that invocation; filenames are not unit identities.",
    context: { before: before.context, after: after.context },
    rootCounts: { before: counts(previous), after: counts(next) },
    ...compareUnits(before.units, after.units),
    rootComparison: compareUnits(previous, next),
  };
}

module.exports = { collectCargoUnits, compareCargoUnits };
