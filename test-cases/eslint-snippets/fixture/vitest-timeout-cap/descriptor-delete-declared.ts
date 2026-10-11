import { defineConfig } from "vitest/config";

declare const declaredCopy: { test: { testTimeout: number } };
declaredCopy.test = { testTimeout: 1 };
delete declaredCopy.test;
defineConfig(declaredCopy);
