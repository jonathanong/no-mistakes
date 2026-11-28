use super::options::parse_options_value;
use no_mistakes::spike_4800::{run_spike, SpikeRunOptions};

pub(crate) fn spike_4800_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<SpikeRunOptions>(options)?;
    let result = run_spike(&options).map_err(|err| napi::Error::from_reason(err.to_string()))?;
    Ok(crate::cli::json_string(&result))
}
