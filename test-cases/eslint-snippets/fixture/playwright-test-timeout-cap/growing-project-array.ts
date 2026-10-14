import { defineConfig } from "@playwright/test";
const grew = [];
grew.push({ timeout: 30001 });
export default defineConfig({ projects: grew });
