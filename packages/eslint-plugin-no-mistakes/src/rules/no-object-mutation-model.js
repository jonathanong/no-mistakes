"use strict";

const syntax = require("./no-object-mutation-syntax");
const values = require("./no-object-mutation-values");

module.exports = {
  ...values,
  ...syntax,
};
