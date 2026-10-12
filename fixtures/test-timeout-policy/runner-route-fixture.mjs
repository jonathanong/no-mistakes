import { fixtureTemplate } from "./fixture-source.mjs";
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
/** Writes real SDK config and a final default-export runner; never constructs it in the fixture. */
export async function prepareRunnerRoute(directory, plugin, route, config) {
    const finalRunner = join(directory, 'final-runner.mts');
    const alternate = join(directory, 'alternate-runner.mts');
    await writeFile(finalRunner, fixtureTemplate("runner-route-fixture-template-10.txt", [route.configTimeout === undefined ? '' : fixtureTemplate("runner-route-fixture-template-11.txt", [route.configTimeout]), route.taskTimeout === undefined ? '' : fixtureTemplate("runner-route-fixture-template-12.txt", [route.taskTimeout]), route.assertDirectNewTarget ? fixtureTemplate("runner-route-fixture-template-13.txt") : '']));
    await writeFile(alternate, fixtureTemplate("runner-route-fixture-template-9.txt"));
    const project = fixtureTemplate("runner-route-fixture-template-5.txt", [route.implicitNative ? '' : fixtureTemplate("runner-route-fixture-template-6.txt", [route.nativeDefault ? 'null' : JSON.stringify(finalRunner)]), route.mergedOverride ? fixtureTemplate("runner-route-fixture-template-7.txt", [JSON.stringify(alternate)]) : '', config, route.nativeDefault ? '' : fixtureTemplate("runner-route-fixture-template-8.txt", [JSON.stringify(finalRunner)])]);
    const header = fixtureTemplate("runner-route-fixture-template-4.txt", [JSON.stringify(plugin)]);
    if (route.standalone) {
        await writeFile(join(directory, 'vitest.standalone.config.mts'), header + fixtureTemplate("runner-route-fixture-template-3.txt", [project]));
        await writeFile(join(directory, 'vitest.config.mts'), fixtureTemplate("runner-route-fixture-template-2.txt"));
    }
    else {
        await writeFile(join(directory, 'vitest.config.mts'), header + fixtureTemplate("runner-route-fixture-template-1.txt", [project]));
    }
    return route.cliOverride ? ['--runner', alternate] : [];
}
