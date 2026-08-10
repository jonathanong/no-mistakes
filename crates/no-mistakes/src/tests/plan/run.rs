use super::{generate_plan, PlanArgs};
use crate::tests::PlanFormat;
use anyhow::Result;
use std::process::ExitCode;

pub(crate) fn run(args: PlanArgs) -> Result<ExitCode> {
    let plan = generate_plan(&args)?;
    let format = if args.json {
        PlanFormat::Json
    } else {
        args.format.unwrap_or(PlanFormat::Json)
    };
    let output = super::super::plan_output::render(&plan, format, "tests plan")?;
    crate::invocation::commit_timeout()?;
    print!("{output}");
    Ok(ExitCode::SUCCESS)
}
