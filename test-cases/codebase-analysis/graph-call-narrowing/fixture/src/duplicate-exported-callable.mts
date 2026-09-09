function decoyLeaf() {}
function actualLeaf() {}

{
  // Same display name as the exported binding; identity must not pick this one.
  function target() {
    decoyLeaf();
  }
}

function target() {
  actualLeaf();
}

const alias = target;
export { alias as run };
export { target };
