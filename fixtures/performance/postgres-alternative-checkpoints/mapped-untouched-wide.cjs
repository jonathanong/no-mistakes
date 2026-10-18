// Same one mapped write as mapped-small, with thirty extra untouched parameters.
function writeMany(flag, target00, target01, target02, target03,
  target04, target05, target06, target07, target08, target09,
  target10, target11, target12, target13, target14, target15, target16, target17, target18, target19, target20, target21, target22, target23, target24,
  target25, target26, target27, target28, target29, target30, target31, target32, target33, target34, target35, target36, target37, target38, target39) {
  flag
    ? ((arguments[1] = "SELECT 1 /* left-00 */"))
    : ((arguments[1] = "SELECT 2 /* right-00 */"));
  return target00;
}
const result = writeMany(true,
  "initial-00", "initial-01", "initial-02", "initial-03", "initial-04",
  "initial-05", "initial-06", "initial-07", "initial-08", "initial-09",
  "initial-10", "initial-11", "initial-12", "initial-13", "initial-14", "initial-15", "initial-16", "initial-17", "initial-18", "initial-19", "initial-20", "initial-21", "initial-22", "initial-23", "initial-24",
  "initial-25", "initial-26", "initial-27", "initial-28", "initial-29", "initial-30", "initial-31", "initial-32", "initial-33", "initial-34", "initial-35", "initial-36", "initial-37", "initial-38", "initial-39");
