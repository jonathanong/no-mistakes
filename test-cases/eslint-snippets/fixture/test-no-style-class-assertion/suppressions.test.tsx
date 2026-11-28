// no-mistakes-disable-next-line test-no-style-class-assertion -- tests className parser
expect(el.className).toContain("bg-primary");
expect(el).toHaveClass("duration-300"); // no-mistakes-disable-line test-no-style-class-assertion -- tests class merger
// no-mistakes-disable-next-line test-no-style-class-assertion
expect(el.classList.contains("top-14")).toBe(true);
// no-mistakes-disable-next-line test-no-style-class-assertion --
expect(el.getAttribute("class")).toMatch(/border-l-2/);
expect(el).not.toHaveClass("sr-only");
