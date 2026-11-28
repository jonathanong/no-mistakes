expect(el).toHaveAttribute("data-state", "open");
expect(el).not.toHaveAttribute("data-state", "closed");
expect(el).toHaveAttribute("aria-expanded", "true");
expect(el).toBeVisible();
expect(el).not.toBeVisible();
expect(el.getAttribute("data-state")).toBe("open");
expect(el.getAttribute(attrName)).toBe("open");
expect(el.getAttribute()).toBeNull();

const cls = el.className;
log(cls);
if (el.className.includes("bg-primary")) {
  log();
}
const hasTop = el.classList.contains("top-14");
log(hasTop);
const rawClass = el.getAttribute("class");
log(rawClass);

expect(parseClasses(el.className)).toEqual(["bg-primary"]);
expect(items.contains("top-14")).toBe(true);
expect(obj.other.contains("top-14")).toBe(true);
expect.assertions(1);
other.soft(el).toHaveClass("duration-300");
other.poll(() => el.className).toContain("bg-primary");
expect.poll(readState).toBe("open");
expect
  .poll(() => {
    log();
    return "open";
  })
  .toBe("open");

function localExpect(expect: any) {
  expect(el).toHaveClass("duration-300");
  expect(el.className).toContain("bg-primary");
}

{
  const expect = createVerifier();
  expect(el).toHaveClass("duration-300");
}
