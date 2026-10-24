// Unrelated nested code must not trigger integration test discovery.
export function render() {
  const nested = () => {
    if (true) {
      return [{ value: 1 }].map(value => value.value);
    }
    return [];
  };
  return nested();
}
