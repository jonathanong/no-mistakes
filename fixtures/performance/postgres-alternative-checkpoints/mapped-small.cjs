// Sloppy script parameters are mapped; keep the ten-parameter baseline fixed.
function writeMany(flag, target00, target01, target02, target03,
  target04, target05, target06, target07, target08, target09) {
  flag
    ? ((arguments[1] = "SELECT 1 /* left-00 */"))
    : ((arguments[1] = "SELECT 2 /* right-00 */"));
  return target00;
}
const result = writeMany(true,
  "initial-00", "initial-01", "initial-02", "initial-03", "initial-04",
  "initial-05", "initial-06", "initial-07", "initial-08", "initial-09");
