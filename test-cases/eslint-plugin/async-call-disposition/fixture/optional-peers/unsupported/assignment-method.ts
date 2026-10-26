// Recovered destructuring trees must not erase a method body.
({ invoke() { callback(); } } = source);
