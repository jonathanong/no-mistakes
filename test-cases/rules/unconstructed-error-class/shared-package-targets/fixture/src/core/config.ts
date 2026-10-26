import { Configured as BaseConfigured } from 'utility-constructors/config';
type Options = ConstructorParameters<typeof BaseConfigured>[0];
type RegisteredKey = 'primary' | 'secondary';
// The constructor annotation narrows arguments without changing class identity.
export const Configured: new (options: Omit<Options, 'key'> & { key: RegisteredKey }) => BaseConfigured = BaseConfigured;
