const Intl = {
  DateTimeFormat: class {},
  NumberFormat() {},
};
new Intl.DateTimeFormat();
Intl.NumberFormat(undefined);
function format(Intl) {
  return Intl.NumberFormat();
}
