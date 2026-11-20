import { defineConfig } from "vite";

export default defineConfig({
  optimizeDeps: {
    include: [
      "widgets > items",
      "orders > items",
      "widgets > extra",
      // no-mistakes-disable-next-line vite-optimize-deps-nested-imports
      "widgets > missing",
      "react",
      "widgets > items > extra",
      "widgets > ./local",
      "foreign > items",
    ],
  },
});
