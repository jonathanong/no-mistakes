expect(el.className).toContain("bg-primary");
expect(el).toHaveClass("duration-300");
expect(el.classList.contains("top-14")).toBe(true);
expect(el.getAttribute("class")).toMatch(/border-l-2/);
