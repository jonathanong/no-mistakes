import { expect } from "vitest";
import { screen } from "@testing-library/react";

export async function asyncQueries() {
  expect(screen.getByText("Save")).toBeDefined();
  expect(screen.getByText("Save")).not.toBeNull();
  expect(screen.getByText("Save")).not.toBeUndefined();
  expect(screen.getAllByRole("button")).toBeDefined();
  expect(screen.getAllByRole("button")).not.toBeNull();

  expect(await screen.findByText("Loaded")).toBeTruthy();
  await expect(screen.findByText("Loaded")).resolves.not.toBeNull();
  expect(screen.findByText("Loaded")).toBeDefined();
  expect(screen.findByText("Loaded")).resolves.toBeDefined();
  expect((void 0, screen.findByText("Seq"))).toBeDefined();

  expect(await screen.findAllByRole("listitem")).toBeTruthy();
  await expect(screen.findAllByRole("listitem")).resolves.toBeDefined();
  expect(screen.findAllByRole("listitem")).not.toBeNull();

  const btn = screen.getByRole("button");
  expect(btn).toBeDefined();
}

export function syncQueries() {
  expect(screen.findByText("Loaded")).toBeDefined();
  expect(screen.findByText("Loaded")).resolves.toBeDefined();
  expect(screen.findAllByRole("listitem")).not.toBeNull();
}

expect(screen.findByText("TopLevel")).toBeDefined();

export function withAssertionCount() {
  expect.assertions(1);
  expect(screen.getByText("Save")).toBeDefined();
}

export async function withHasAssertions() {
  expect.hasAssertions();
  expect(screen.findByText("Loaded")).toBeDefined();
}
