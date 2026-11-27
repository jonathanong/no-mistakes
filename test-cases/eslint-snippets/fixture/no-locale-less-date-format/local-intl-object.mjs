const Intl = { DateTimeFormat() { return { format() {} }; } };
new Intl.DateTimeFormat();

function localFactory() {
  function require() {
    return { Intl: { DateTimeFormat() {} } };
  }
  const Intl = require().Intl;
  Intl.DateTimeFormat();
}
