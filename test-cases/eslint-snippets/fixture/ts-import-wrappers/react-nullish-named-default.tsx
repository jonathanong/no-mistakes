import { default as R } from "react";
import { "default" as Quoted } from "react";

export function named(child: R.ReactNode) {
  return child ?? null;
}

export function quoted(child: Quoted.ReactNode) {
  return child ?? null;
}
