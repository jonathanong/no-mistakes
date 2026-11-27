import * as Intl from "@example/intl";
import IntlEquals = require("@example/intl");
let IntlRequired = require("@example/intl");

// no-mistakes-disable-next-line no-locale-less-date-format -- module is outside this file
new Intl.DateTimeFormat();
new Intl.DateTimeFormat();

// no-mistakes-disable-next-line no-locale-less-date-format -- module is outside this file
new IntlEquals.DateTimeFormat();
new IntlEquals.DateTimeFormat();

// no-mistakes-disable-next-line no-locale-less-date-format -- module is outside this file
new IntlRequired.DateTimeFormat();
new IntlRequired.DateTimeFormat();
const IntlMember = require("@example/intl").Intl;
new IntlMember.DateTimeFormat();

function localAssignment() {
  let Intl = require("@example/intl");
  Intl.DateTimeFormat();
  Intl = { DateTimeFormat() {} };
  Intl.DateTimeFormat();
}

function branchAssignment(useLocal: boolean) {
  let Intl = require("@example/intl");
  if (useLocal) Intl = { DateTimeFormat() {} };
  Intl.DateTimeFormat();
}

declare const key: string;
new Intl[key]("en-GB");

function redeclaredNamespace() {
  var Intl = require("@example/intl");
  var Intl;
  Intl.DateTimeFormat();
}

function uninitializedNamespace() {
  var Intl;
  Intl.DateTimeFormat();
}

function initializedNamespace() {
  var Intl = require("@example/intl");
  var Intl = { DateTimeFormat() {} };
  Intl.DateTimeFormat();
}
