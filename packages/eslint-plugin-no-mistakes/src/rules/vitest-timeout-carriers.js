"use strict";
function createCarriers(option, check) {
  function configCheck(entry, ...args) {
    const unresolved = entry.value.kind !== "number" || entry.value.runtimeFunction;
    if (unresolved && option.configUnknownValues === "defer-to-runner") return;
    check(entry, ...args);
  }
  function properties(
    value,
    names,
    max,
    origin,
    carrier = "timeout options",
    carrierOrigin = origin,
    inspect = check,
  ) {
    if (value.kind !== "object") {
      inspect({ value: { kind: "unknown" }, origin }, carrier, max, origin);
      return;
    }
    let unresolvedKeys = false;
    for (const name of names) {
      const entry = value.properties.get(name);
      if (entry) inspect(entry, name, max, origin);
      else if (value.unknown) unresolvedKeys = true;
    }
    if (unresolvedKeys)
      inspect({ value: { kind: "unknown" }, origin: carrierOrigin }, carrier, max, carrierOrigin);
  }
  function defaults(value, project, origin = project) {
    const max = option.defaultMax ?? 5000;
    if (value.kind !== "object") {
      configCheck({ value: { kind: "unknown" }, origin }, "config carrier", max, origin);
      return;
    }
    const testEntry = value.properties.get("test");
    if (!testEntry) {
      if (value.unknown)
        configCheck({ value: { kind: "unknown" }, origin }, "config carrier", max, origin);
      return;
    }
    const test = testEntry.value;
    const testOrigin = project || testEntry.origin || origin;
    properties(
      test,
      ["testTimeout", "hookTimeout"],
      max,
      test.kind === "object" ? project : testOrigin,
      "test options carrier",
      testOrigin,
      configCheck,
    );
    if (test.kind !== "object") return;
    const projects = test.properties.get("projects");
    if (projects) {
      if (projects.value.kind === "array") {
        if (!projects.value.lengthKnown)
          configCheck(
            { value: { kind: "unknown" }, origin: projects.origin },
            "projects carrier",
            max,
            projects.origin,
          );
        for (const entry of projects.value.entries)
          defaults(entry.value, entry.origin, entry.origin);
      } else
        configCheck(
          { value: { kind: "unknown" }, origin: projects.origin },
          "projects carrier",
          max,
          projects.origin,
        );
    } else if (test.unknown && !project)
      configCheck(
        { value: { kind: "unknown" }, origin: testOrigin },
        "projects carrier",
        max,
        testOrigin,
      );
  }
  return { properties, defaults };
}
module.exports = { createCarriers };
