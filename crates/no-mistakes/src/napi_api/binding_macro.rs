// Keep JSON N-API entrypoints asynchronous while making their Rust and
// JavaScript names explicit at the registration site. Defined before the
// child modules so their registrations use the same declarative boundary.
macro_rules! json_binding {
    ($rust_name:ident, $js_name:literal, $implementation:path, independent) => {
        #[cfg(not(coverage))]
        #[cfg_attr(not(test), napi(js_name = $js_name))]
        pub fn $rust_name(
            env: napi::Env,
            options_json: napi::bindgen_prelude::Buffer,
        ) -> napi::Result<napi::bindgen_prelude::Object<'static>> {
            async_task::independent::json(env, options_json, $implementation)
        }
    };
    ($rust_name:ident, $js_name:literal, $implementation:path, pure) => {
        #[cfg(not(coverage))]
        #[cfg_attr(not(test), napi(js_name = $js_name))]
        pub fn $rust_name(
            options_json: napi::bindgen_prelude::Buffer,
        ) -> AsyncTask<async_task::PureJsonTask> {
            AsyncTask::new(async_task::PureJsonTask::new(options_json, $implementation))
        }
    };
    ($rust_name:ident, $js_name:literal, $implementation:path) => {
        json_binding!($rust_name, $js_name, $implementation, value);
    };
    ($rust_name:ident, $js_name:literal, $implementation:path, value) => {
        #[cfg(not(coverage))]
        #[cfg_attr(not(test), napi(js_name = $js_name))]
        pub fn $rust_name(options_json: napi::bindgen_prelude::Buffer) -> AsyncTask<JsonValueTask> {
            AsyncTask::new(JsonValueTask::new(options_json, $implementation))
        }
    };
    ($rust_name:ident, $js_name:literal, $implementation:path, text) => {
        #[cfg(not(coverage))]
        #[cfg_attr(not(test), napi(js_name = $js_name))]
        pub fn $rust_name(options_json: napi::bindgen_prelude::Buffer) -> AsyncTask<JsonValueTask> {
            AsyncTask::new(JsonValueTask::new_text(options_json, $implementation))
        }
    };
    ($rust_name:ident, $js_name:literal, $implementation:path, string) => {
        #[cfg(not(coverage))]
        #[cfg_attr(not(test), napi(js_name = $js_name))]
        pub fn $rust_name(
            options_json: napi::bindgen_prelude::Buffer,
        ) -> AsyncTask<async_task::JsonTask> {
            AsyncTask::new(async_task::JsonTask::new(options_json, $implementation))
        }
    };
}
