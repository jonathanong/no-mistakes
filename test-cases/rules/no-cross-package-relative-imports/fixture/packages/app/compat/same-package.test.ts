import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { vi } from "vitest";
import wav from "./fixtures/sample.wav";
import logo from "./assets/logo.png";
import icon from "./assets/icon.svg";
import styles from "./styles/theme.css";
import { missingLocal } from "./missing-local.mts";

export const assets = [wav, logo, icon, styles, missingLocal];

async function loadSamePackageFixtures(dir: string, file: string, specifier: string) {
  const actualA = await vi.importActual<typeof import("./a.ts")>("./a.ts");
  const actualMod = await vi.importActual<typeof import("../mod")>("../mod");
  const actualRaw = await vi.importActual("./a.ts");
  const mockedMod = await vi.importMock("../mod");
  const sibling = await import(pathToFileURL(join(dir, file)).href);
  const dynamic = await import(specifier);
  return { actualA, actualMod, actualRaw, mockedMod, sibling, dynamic };
}

export { loadSamePackageFixtures };

export const workspaceActual = vi.importActual("@sample/shared");
