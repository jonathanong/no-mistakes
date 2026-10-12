export class ProcessGroupDrainTimeoutError extends Error {
    constructor(processGroupId, timeoutMs) {
        super(`Process group ${processGroupId} did not exit within ${timeoutMs}ms after SIGKILL`);
        this.name = 'ProcessGroupDrainTimeoutError';
    }
}
export function isProcessGroupAlive(processGroupId) {
    validateProcessGroupId(processGroupId);
    assertSupportedPlatform();
    try {
        process.kill(-processGroupId, 0);
        return true;
    }
    catch (error) {
        return error.code !== 'ESRCH';
    }
}
export async function waitForProcessGroupExit(processGroupId, timeoutMs) {
    validateProcessGroupId(processGroupId);
    validateTimerDelay(timeoutMs);
    assertSupportedPlatform();
    const deadline = performance.now() + timeoutMs;
    while (isProcessGroupAlive(processGroupId)) {
        if (performance.now() >= deadline)
            throw new ProcessGroupDrainTimeoutError(processGroupId, timeoutMs);
        await new Promise((resolve) => setTimeout(resolve, Math.min(10, deadline - performance.now())));
    }
}
function assertSupportedPlatform() {
    if (process.platform === 'win32')
        throw new Error('process-group control is unsupported on Windows');
}
function validateProcessGroupId(processGroupId) {
    if (!Number.isSafeInteger(processGroupId) ||
        processGroupId <= 1 ||
        processGroupId > 2_147_483_647)
        throw new RangeError('processGroupId must be a positive supported PID');
}
function validateTimerDelay(timeoutMs) {
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0 || timeoutMs > 2_147_483_647)
        throw new RangeError('timeoutMs must be a positive Node timer delay');
}
