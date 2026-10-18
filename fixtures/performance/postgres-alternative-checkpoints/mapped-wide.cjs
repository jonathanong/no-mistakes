// Sloppy script parameters are mapped; keep the ten-parameter baseline fixed.
function writeMany(flag, target00, target01, target02, target03,
  target04, target05, target06, target07, target08, target09) {
  flag
    ? ((arguments[1] = "SELECT 1 /* left-00 */"), (arguments[2] = "SELECT 1 /* left-01 */"), (arguments[3] = "SELECT 1 /* left-02 */"), (arguments[4] = "SELECT 1 /* left-03 */"), (arguments[5] = "SELECT 1 /* left-04 */"), (arguments[6] = "SELECT 1 /* left-05 */"), (arguments[7] = "SELECT 1 /* left-06 */"), (arguments[8] = "SELECT 1 /* left-07 */"), (arguments[9] = "SELECT 1 /* left-08 */"), (arguments[10] = "SELECT 1 /* left-09 */"))
    : ((arguments[1] = "SELECT 2 /* right-00 */"), (arguments[2] = "SELECT 2 /* right-01 */"), (arguments[3] = "SELECT 2 /* right-02 */"), (arguments[4] = "SELECT 2 /* right-03 */"), (arguments[5] = "SELECT 2 /* right-04 */"), (arguments[6] = "SELECT 2 /* right-05 */"), (arguments[7] = "SELECT 2 /* right-06 */"), (arguments[8] = "SELECT 2 /* right-07 */"), (arguments[9] = "SELECT 2 /* right-08 */"), (arguments[10] = "SELECT 2 /* right-09 */"));
  return target09;
}
const result = writeMany(true,
  "initial-00", "initial-01", "initial-02", "initial-03", "initial-04",
  "initial-05", "initial-06", "initial-07", "initial-08", "initial-09");
