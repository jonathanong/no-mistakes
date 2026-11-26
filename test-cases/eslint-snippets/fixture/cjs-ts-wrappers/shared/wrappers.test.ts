import { test, beforeEach } from "vitest";
const items = [] as string[];
test("collects", () => { items.push("x"); });
let list: string[] = [];
beforeEach(() => { list = [] as string[]; });
test("resets", () => { list.push("y"); });
