// This helper intentionally lives outside the inherited project's root. A
// dynamic setup declaration must still react when the helper changes.
export const dynamicSetup = process.env.SETUP_FILE
