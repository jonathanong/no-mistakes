import { resetPolicy } from "./settings";

export default {
  test: { include: ["tests/**/*.test.ts"], setupFiles: ["./setup.ts"], ...resetPolicy },
};
