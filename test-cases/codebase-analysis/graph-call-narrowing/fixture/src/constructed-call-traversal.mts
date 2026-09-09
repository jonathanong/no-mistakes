function constructorLeaf() {}
function unusedLeaf() {}

class Service {
  constructor() {
    constructorLeaf();
  }

  unused() {
    unusedLeaf();
  }
}

export function run() {
  new Service();
}
