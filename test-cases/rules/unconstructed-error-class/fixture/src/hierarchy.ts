// Satisfied: ChildError extends it.
export abstract class AppError extends Error {}

// Constructed in src/use.ts.
export class ChildError extends AppError {}

// Satisfied: Grandchild extends it, even though Grandchild is dead.
export class MiddleError extends ChildError {}

// Flagged: an error class two levels below Error that nothing constructs.
export class Grandchild extends MiddleError {}

// Flagged: an abstract base that nothing subclasses.
export abstract class OrphanBase extends Error {}
