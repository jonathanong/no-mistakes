const [{ open } = other.panel] = [];
function nested([{ open: shown } = other.panel]) {
  return shown;
}
// An identifier default is not an object pattern.
const [label = other.panel] = [];
