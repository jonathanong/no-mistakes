import Real = require("react");
import React = require("./ui");

export function realReact(value: Real.ReactNode) {
  return value ?? null;
}

export function foreignReact(value: React.ReactNode) {
  return value ?? null;
}
