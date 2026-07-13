use super::{Ctx, ImportBinding, Options};
use anyhow::Result;

pub(super) fn imported_member_options(
    import: &ImportBinding,
    member: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    super::members::imported_member_options_from(import, member, ctx.path, ctx)
}
