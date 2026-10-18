// The outer pattern and the default reach the same property and chain once.
const { panel: { open } = other.panel } = other;
// A different property is still reported.
const { drawer: { toggle } = other.drawer } = other;
