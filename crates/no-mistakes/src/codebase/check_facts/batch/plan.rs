use super::CheckFactPlan;

pub(super) struct DemandPlan<'a> {
    base: &'a CheckFactPlan,
    owned: Option<Box<CheckFactPlan>>,
}

impl<'a> DemandPlan<'a> {
    pub(super) fn borrowed(base: &'a CheckFactPlan) -> Self {
        Self { base, owned: None }
    }

    pub(super) fn as_ref(&self) -> &CheckFactPlan {
        self.owned.as_deref().unwrap_or(self.base)
    }

    pub(super) fn make_mut<const COUNT: bool>(&mut self, clones: &mut usize) -> &mut CheckFactPlan {
        self.owned
            .get_or_insert_with(|| Box::new(clone_plan::<COUNT>(self.base, clones)))
    }
}

pub(super) fn clone_plan<const COUNT: bool>(
    plan: &CheckFactPlan,
    clones: &mut usize,
) -> CheckFactPlan {
    if COUNT {
        *clones += 1;
    }
    plan.clone()
}
