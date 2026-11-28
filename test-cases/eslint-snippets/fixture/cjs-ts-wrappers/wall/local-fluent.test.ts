const start = 1;

function check(value: number) {
  return {
    toBeLessThan(limit: number) {
      void value;
      void limit;
    },
  };
}

check(Date.now() - start).toBeLessThan(10);
