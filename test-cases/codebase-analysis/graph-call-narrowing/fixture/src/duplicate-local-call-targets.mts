function firstLeaf() {}
function secondLeaf() {}

export function run() {
  {
    function target() {
      firstLeaf();
    }
    target();
  }
  {
    function target() {
      secondLeaf();
    }
    target();
  }
}
