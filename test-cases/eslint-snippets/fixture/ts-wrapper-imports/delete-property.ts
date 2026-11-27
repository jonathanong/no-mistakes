const obj = {
  foo: 1,
  fn: <T>(value: T) => value,
};

delete (obj.foo satisfies number);
delete obj.fn<number>;
