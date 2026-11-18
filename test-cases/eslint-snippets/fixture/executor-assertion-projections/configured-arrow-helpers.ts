export const checkArrow: (value: unknown) => void = (value) => {
  if (!value) throw "Missing input";
};

export const checkFunction: (value: unknown) => void = function (value) {
  if (!value) throw "Missing input";
};
