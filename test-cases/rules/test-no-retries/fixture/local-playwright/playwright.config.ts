import { defineConfig } from "@playwright/test";
import { createBrowserOptions } from "./config/browser-options";

export default defineConfig(createBrowserOptions());
