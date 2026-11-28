async function checkNegated(el: HTMLElement, findEl: () => Promise<HTMLElement>) {
  expect(el.className).not.toContain("bg-primary");
  expect(el).not.toHaveClass("duration-300");
  expect(el).not.toHaveClass("hidden");
  expect(el.classList.contains("top-14")).not.toBe(true);
  expect(el.getAttribute("class")).not.toMatch(/border-l-2/);
  await expect(findEl()).resolves.toHaveClass("duration-300");
  await expect(findEl()).resolves.not.toHaveClass("duration-300");
}
