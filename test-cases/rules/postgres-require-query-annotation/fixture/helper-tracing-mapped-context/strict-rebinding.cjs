"use strict";
// Strict functions keep the call-time argument value despite parameter reassignment.
function forward(statement) {
  var statement = "SELECT 1";
  return arguments[0];
}
const result = forward("/* initial annotation */ SELECT 1");
