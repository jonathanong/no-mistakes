import type {} from "@testing-library/jest-dom";
import { toBeVisible } from "@testing-library/jest-dom/matchers";
import { expect } from "vitest";
import { screen } from "@testing-library/react";

void toBeVisible;
expect(screen.getByText("Save")).toBeDefined();
