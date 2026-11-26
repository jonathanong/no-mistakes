import type React from "react";
import { Button as RawButton } from "./ui";

const ActionButton = RawButton as React.ComponentType;
const SatisfiedButton = RawButton satisfies React.ComponentType;
const NonNullButton = RawButton!;
const InstButton = RawButton<string>;

export const C = () => (
  <>
    <div role="button" />
    <div role={"button" as const} />
    <div role={"button" satisfies string} />
    <div role={"button"!} />
    <ActionButton />
    <SatisfiedButton />
    <NonNullButton />
    <InstButton />
    <ActionButton data-pw="ok" />
  </>
);
