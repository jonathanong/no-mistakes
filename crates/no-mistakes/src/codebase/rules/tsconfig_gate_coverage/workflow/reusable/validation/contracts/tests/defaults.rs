use super::{on, workflow_call_shape_valid};

#[test]
fn workflow_call_input_defaults_require_available_typed_expressions() {
    for default in [
        "enabled:\n      type: boolean\n      default: '${{ inputs.enabled }}'",
        "attempts:\n      type: number\n      default: '${{ 2 }}'",
        "enabled:\n      type: boolean\n      default: \"${{ fromJSON('false') }}\"",
        "attempts:\n      type: number\n      default: \"${{ fromJSON('0') }}\"",
        "label:\n      type: string\n      default: 'release-${{ github.ref_name }}'",
        "label:\n      type: string\n      default: '${{ vars.LABEL }}'",
    ] {
        assert!(
            workflow_call_shape_valid(Some(&on(&format!(
                "workflow_call:\n  inputs:\n    {default}"
            )))),
            "{default}"
        );
    }

    for default in [
        "label:\n      type: string\n      default: '${{ }}'",
        "enabled:\n      type: boolean\n      default: '${{ secrets.TOKEN }}'",
        "enabled:\n      type: boolean\n      default: \"${{ 'true' }}\"",
        "label:\n      type: string\n      default: '${{ true }}'",
        "attempts:\n      type: number\n      default: \"${{ contains('x', 'x') }}\"",
        "label:\n      type: string\n      default: \"${{ fromJSON('{}') }}\"",
        "enabled:\n      type: boolean\n      default: \"${{ fromJSON('not-json') }}\"",
        "enabled:\n      type: boolean\n      default: \"${{ contains(fromJSON('not-json'), 'x') }}\"",
    ] {
        assert!(
            !workflow_call_shape_valid(Some(&on(&format!(
                "workflow_call:\n  inputs:\n    {default}"
            )))),
            "{default}"
        );
    }
}
