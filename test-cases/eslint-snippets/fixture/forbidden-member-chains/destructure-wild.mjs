{
  const { entry } = call();
}
{
  const { entry } = records;
}
{
  const { entry } = other.records;
}
// A parameter named records does not hide the forbidden spelling.
function f(records) {
  const { entry } = records;
  return records.entry;
}
