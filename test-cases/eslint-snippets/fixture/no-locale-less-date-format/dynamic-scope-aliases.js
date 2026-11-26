const key = "toLocaleDateString";
const Format = Intl.DateTimeFormat;
with (overrides) {
  date[key]();
  new Format();
}
