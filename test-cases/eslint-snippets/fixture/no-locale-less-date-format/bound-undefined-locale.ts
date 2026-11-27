import { locale as undefined } from "@example/locales";
import * as Intl from "@example/intl";

new Date().toLocaleString(undefined);
new Intl.DateTimeFormat(undefined);
new Intl.DateTimeFormat();

function withLocalLocale() {
  const undefined = "en-GB";
  new Intl.DateTimeFormat(undefined);
}
