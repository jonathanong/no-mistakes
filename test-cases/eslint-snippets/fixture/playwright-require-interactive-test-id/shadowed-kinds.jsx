import { Button as Save } from "./ui.jsx";
// These local bindings reuse a renamed import without inheriting its meaning.
function Wrapper(Save) { return <Save />; }
function WithFunction() {
  function Save() { return <span />; }
  return <Save />;
}
function WithClass() {
  class Save {}
  return <Save />;
}
