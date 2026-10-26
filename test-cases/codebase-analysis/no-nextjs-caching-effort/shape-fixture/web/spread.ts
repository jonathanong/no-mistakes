// A spread can supply options even without a second syntactic argument.
declare const args: [string, RequestInit];
fetch(...args);
fetch(...['/spread', { cache: 'force-cache' }]);
fetch('/without-options');
