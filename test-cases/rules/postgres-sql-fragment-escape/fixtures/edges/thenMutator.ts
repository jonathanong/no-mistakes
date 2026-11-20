export function extend(extra: unknown) {
  const pending = import("./widgetColumns");
  pending.then((ns) => {
    ns.widgetColumns.append(extra);
  });
}
