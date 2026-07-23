import { projectFactsMarker } from '../project-list'

export const unit = (): string => (projectFactsMarker ? 'unit' : 'unreachable')
