declare function work(): Promise<void>;

const obj = {
  cleanup() {
    return Promise.resolve();
  },
};

work()["catch" as const](() => {});
obj["cleanup" as const]().catch(() => {});
