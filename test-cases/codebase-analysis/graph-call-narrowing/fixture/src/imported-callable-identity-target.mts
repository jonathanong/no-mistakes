export function actual() {
  actualLeaf();
}

function actualLeaf() {}

// It deliberately collides with the consumer's local import spelling.
function aliasName() {}
