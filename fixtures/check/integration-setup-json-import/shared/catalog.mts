// Reproduces a downstream monorepo's setup helper that imports a JSON
// catalog with an import attribute. This is not `.d.ts`, so it must remain a
// runtime trigger candidate for the setup-dependency walk, but it must never
// be handed to the TS/JS parser: `unsupported JavaScript/TypeScript file`.
import aliases from '../localization/catalog/aliases.json' with { type: 'json' }

export function registerAliases() {
  return aliases
}
