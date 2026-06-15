// Mock/stub test that the vitest suite `exclude`s from the normal run, but that
// `tests impact` must still surface via `tests.impact.alwaysIncludeTests`.
import { widget } from './widget.mts';

widget();
