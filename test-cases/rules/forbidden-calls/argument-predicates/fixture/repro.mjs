import { execFile as run } from 'node:child_process'
import * as processes from 'node:child_process'

const repository = '/workspace'
const fixtureRoot = '/tmp/analyzer-fixture'
const cli = '/workspace/node_modules/no-mistakes/bin/no-mistakes.js'
const fixtureArgs = [cli, 'check', '--root', fixtureRoot, '--format', 'json']
const liveArgs = [cli, 'check', '--format', 'json']
const cwd = repository
const liveOptions = { cwd }

// Synthetic analysis inputs only: never execute these process launches.
run(process.execPath, fixtureArgs, { cwd: fixtureRoot }, () => {})
run(process.execPath, liveArgs, liveOptions, () => {})
processes.execFile(process.execPath, [cli, 'check'], { cwd: repository }, () => {})
run(process.execPath, [cli, 'check', '--root', fixtureRoot], { cwd: repository }, () => {})
