import { UsedError as BaseError, UsedTwin as BaseTwin } from '../errors';
// Typed aliases must credit these exact classes, never their unused siblings.
export const ConstructedError: new () => BaseError = BaseError;
export const ConstructedTwin: new () => BaseTwin = BaseTwin;
