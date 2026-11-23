import { test } from "vitest";
const load = (id: string) => id;
test("profile fixture", () => load("profile.avatar"));
