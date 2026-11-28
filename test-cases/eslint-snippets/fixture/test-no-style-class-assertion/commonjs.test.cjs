const { expect } = require("@jest/globals");
const vitest = require("vitest");

const softExpect = expect.soft;
expect(el.className).toContain("bg-primary");
softExpect(el).toHaveClass("duration-300");
vitest.expect(el.classList.contains("top-14")).toBe(true);
require("vitest").expect(el.getAttribute("class")).toMatch(/border-l-2/);
