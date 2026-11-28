import { expect } from "vitest";
import * as rtl from "@testing-library/react";
import { render, screen, within } from "@testing-library/react";
import { getByText as domGetByText } from "@testing-library/dom";

declare const ui: unknown;
declare const dialog: HTMLElement;

export async function throwingQueries() {
  // Positive cases (throwing-query)
  expect(screen.getByText("Save")).toBeDefined();
  expect(screen.getByText("Save")).not.toBeUndefined();
  expect(screen.getByText("Save")).not.toBeNull();
  expect(screen.getByText("Save")).toBeTruthy();
  expect(screen.getByText("Save")).not.toBeFalsy();
  expect(screen.getAllByRole("button")).toBeDefined();
  expect(screen.getAllByRole("button")).not.toBeNull();
  expect(await screen.findByText("Loaded")).toBeTruthy();
  expect(await screen.findByText("Loaded")).not.toBeNull();
  await expect(screen.findByText("Loaded")).resolves.toBeDefined();
  await expect(screen.findByText("Loaded")).resolves.not.toBeNull();
  await expect(screen.findAllByRole("listitem")).resolves.not.toBeUndefined();

  const { getByText, findAllByRole } = render(ui);
  expect(getByText("Title")).toBeDefined();
  expect(await findAllByRole("listitem")).toBeTruthy();

  const view = render(ui);
  expect(view.getByText("Title")).toBeDefined();
  expect(render(ui).getByText("Title")).toBeDefined();

  expect(within(dialog).getByRole("button")).toBeDefined();
  const scoped = within(dialog);
  await expect(scoped.findByText("Hi")).resolves.toBeDefined();
  const { getByLabelText: getLabel } = within(dialog);
  expect(getLabel("Email")).toBeTruthy();

  expect(rtl.screen.getByText("NS")).toBeDefined();
  expect(domGetByText(dialog, "Raw")).toBeDefined();

  const button = screen.getByRole("button");
  expect(button).toBeDefined();

  const rtlAlias = rtl;
  expect(rtlAlias.screen.getByText("Alias")).toBeDefined();
  const fallback = (t: string) => dialog;
  const { getByText: getT = fallback, ...rest } = screen;
  void rest;
  expect(getT("Default")).toBeDefined();
  const { ["getByText"]: computedGet } = screen;
  expect(computedGet("Computed")).toBeDefined();
  const queryFn = screen.getByText;
  expect(queryFn("Fn")).toBeDefined();

  // Counterexamples (allowed)
  expect(screen.getByText("Save")).toHaveTextContent("Save");
  expect(screen.queryByText("Missing")).toBeTruthy();
  expect(screen.queryByText("Missing")).not.toBeNull();
  await expect(screen.findByText("Fail")).rejects.toBeDefined();

  const localScreen = { getByText: (_text: string): unknown => undefined };
  expect(localScreen.getByText("Local")).toBeDefined();
  const { getByText: localGet } = localScreen;
  expect(localGet("Local")).toBeDefined();
  const { ...restQueries } = screen as unknown as { (): unknown };
  expect(restQueries()).toBeDefined();
  const asyncView = await render(ui);
  expect(asyncView.getByText("Awaited")).toBeDefined();
  const getFactory = () => localScreen;
  expect(getFactory().getByText("Factory")).toBeDefined();
  const makeFn = () => () => undefined as unknown;
  expect(makeFn()()).toBeDefined();
}
