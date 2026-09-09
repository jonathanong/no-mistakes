import { setTimeout as wait } from "node:timers";

declare const vi: {
  mock: (specifier: string) => void;
  fn: () => void;
};

// Unit-selected mocks must not leak into the integration-no-mocks application.
vi.mock("./timers.test.mts");
vi.fn();
wait(() => {}, 1);
