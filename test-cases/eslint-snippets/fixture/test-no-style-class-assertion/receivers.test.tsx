async function checkReceivers(container: HTMLElement, el: HTMLElement) {
  expect((container.firstChild as HTMLElement).className).toContain("bg-primary");
  expect(container.firstChild?.className).toContain("bg-primary");
  expect(container.firstChild).toHaveClass("duration-300");
  expect((container.firstChild as Element).classList.contains("top-14")).toBe(true);
  expect(container.firstChild!.getAttribute("class")).toMatch(/border-l-2/);
  expect(screen.getByRole("button").className).toContain("bg-primary");
  expect(screen.getByRole("button")).toHaveClass("duration-300");
  expect(screen.getByRole("button").classList.contains("top-14")).toBe(true);
  expect(screen.getByRole("button").getAttribute("class")).toMatch(/border-l-2/);
  expect.soft(el).toHaveClass("duration-300");
  expect.soft(el.className).toContain("bg-primary");
  await expect.poll(() => el).toHaveClass("duration-300");
  await expect.poll(() => el.className).toContain("bg-primary");
  await expect
    .poll(() => {
      return el.classList.contains("top-14");
    })
    .toBe(true);
  await expect
    .poll(function () {
      return el.getAttribute("class");
    })
    .toMatch(/border-l-2/);

  const cls = el.className;
  expect(cls).toContain("bg-primary");
  const classes = el.classList;
  expect(classes.contains("top-14")).toBe(true);
  const hasTop = el.classList.contains("top-14");
  expect(hasTop).toBe(true);
  const attr = "class";
  expect(el.getAttribute(attr)).toMatch(/border-l-2/);
  const classAttr = el.getAttribute("class");
  expect(classAttr).toContain("p-4");
}
