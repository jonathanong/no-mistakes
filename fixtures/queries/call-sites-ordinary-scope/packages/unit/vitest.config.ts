import { subject } from './src/subject'
subject('vitest config caller')
import { settings } from './runner-settings'
// Evaluating the runner config reads this helper before parallel fact collection.
export default settings
