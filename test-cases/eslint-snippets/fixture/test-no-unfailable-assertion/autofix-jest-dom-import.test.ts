import "@testing-library/jest-dom/vitest";
import { expect } from "vitest";
import { screen } from "@testing-library/react";

export async function asyncWithJestDom() {
  expect(screen.getByText("Save")).toBeDefined();
  expect(screen.getByText("Save")).not.toBeNull();
  expect(screen.getByText("Save")).not.toBeUndefined();

  const btn = screen.getByRole("button");
  expect(btn).toBeDefined();

  const awaited = await screen.findByText("Ready");
  expect(awaited).toBeTruthy();

  const pending = screen.findByText("Pending");
  expect(pending).toBeDefined();

  expect(await screen.findByText("Loaded")).toBeTruthy();
  await expect(screen.findByText("Loaded")).resolves.not.toBeNull();
  expect(screen.findByText("Loaded")).toBeDefined();
  expect(screen.findByText("Loaded")).resolves.toBeDefined();

  expect(screen.getAllByRole("button")).toBeDefined();
  expect(await screen.findAllByRole("listitem")).not.toBeNull();
  await expect(screen.findAllByRole("listitem")).resolves.toBeDefined();
}

export function syncWithJestDom() {
  expect(screen.findByText("Loaded")).toBeDefined();
  expect(screen.findByText("Loaded")).resolves.toBeDefined();
}
