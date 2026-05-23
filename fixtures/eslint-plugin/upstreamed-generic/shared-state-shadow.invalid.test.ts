let mockData = {};

vi.mock("shared/data", () => {
  const mockData = {};
  return { data: mockData };
});

it("reports module state when vi.mock only references a shadow", () => {
  mockData = { next: true };
});
