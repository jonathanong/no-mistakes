export const arrowExpr = (t: unknown) => t;

export const arrowBlock = (t: unknown) => {
  {
    return t;
  }
};

export const fnExpr = function (t: unknown, flag: boolean) {
  if (flag) {
    return t;
  } else {
    return t;
  }
};
