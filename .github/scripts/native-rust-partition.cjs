function partition(cases, index, count) {
  const identity = ({ executable, name }) => `${executable}\0${name}`;
  return [...cases]
    .sort((left, right) => {
      const a = identity(left);
      const b = identity(right);
      return a < b ? -1 : a > b ? 1 : 0;
    })
    .filter((_, offset) => offset % count === index - 1);
}

module.exports = { partition };
