// An implicit arguments object has no legacy declaration to receive metadata here.
function ordinary(seed = arguments) { return arguments; }
const arrow = () => arguments;
const arrowBody = (seed = arguments) => { var arguments = 'body'; return arguments; };
