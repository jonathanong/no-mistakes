import type { ReactNode } from "react";

declare const key: string;

export function numericMember(value: { 0: ReactNode }) {
  return value[0] ?? null;
}

export function stringMember(value: { 0: ReactNode }) {
  return value["0"] ?? null;
}

export function dynamicMember(value: { 0: ReactNode }) {
  return value[key] ?? null;
}

export function numericDestructure(p: { 0: ReactNode }) {
  const { [0]: child } = p;
  return child ?? null;
}

export function numericParam({ [0]: child }: { 0: ReactNode }) {
  return child ?? null;
}

export function dynamicDestructure(p: { 0: ReactNode }) {
  const { [key]: child } = p;
  return child ?? null;
}

export function numericType(value: { [0]: ReactNode }) {
  return value[0] ?? null;
}

export function assertedNumericMember(value: { 0: ReactNode }) {
  return value[0 as const] ?? null;
}

export function assertedNumericDestructure(p: { 0: ReactNode }) {
  const { [0 as const]: child } = p;
  return child ?? null;
}

export function assertedDynamicMember(value: { 0: ReactNode }) {
  return value[key as const] ?? null;
}
