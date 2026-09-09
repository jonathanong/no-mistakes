declare const vi: {
  mock: (specifier: string) => void;
  doMock: (specifier: string) => void;
  importMock: (specifier: string) => void;
  fn: () => void;
  spyOn: (target: object, method: string) => void;
  stubGlobal: (name: string, value: unknown) => void;
};
declare const jest: {
  mock: (specifier: string) => void;
  doMock: (specifier: string) => void;
};

vi.mock("./timers.test.mts");
vi.doMock("./timers.test.mts");
vi.importMock("./timers.test.mts");
vi.fn();
vi.spyOn(console, "log");
vi.stubGlobal("fetch", () => undefined);
jest.mock("./timers.test.mts");
jest.doMock("./timers.test.mts");
