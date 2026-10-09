function indexCases(key, other) {
  const zero = (arguments[0] = "zero");
  const numeric = (arguments[1] = "numeric");
  const quoted = (arguments["2"] = "quoted");
  const fractional = (arguments[1.5] = "fractional");
  const overflow = (arguments[9007199254740992] = "overflow");
  const huge = (arguments[1e100] = "huge");
  const noncanonical = (arguments["01"] = "noncanonical");
  const nonnumeric = (arguments["extra"] = "nonnumeric");
  const dynamic = (arguments[key] = "dynamic");
  const otherReceiver = (other[0] = "other receiver");
  return [
    zero,
    numeric,
    quoted,
    fractional,
    overflow,
    huge,
    noncanonical,
    nonnumeric,
    dynamic,
    otherReceiver,
  ];
}
