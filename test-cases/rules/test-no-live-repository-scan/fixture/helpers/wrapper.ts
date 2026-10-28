import { policyNames } from "./policy-scan";

export function loadPolicy(): number {
  return policyNames().length;
}
