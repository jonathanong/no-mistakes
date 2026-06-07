"use strict";

const assets = require("./assets");
const download = require("./download");
const installer = require("./installer");
const platform = require("./platform");
const retry = require("./retry");

module.exports = {
  ...assets,
  ...download,
  ...installer,
  ...platform,
  ...retry,
};
