//! Promise polling has its own bounded worker, independent of libuv analyses.
use std::sync::mpsc;

struct Executor<Job> {
    sender: mpsc::SyncSender<Job>,
}

impl<Job: Send + 'static> Executor<Job> {
    fn start(capacity: usize, run: impl Fn(Job) + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        std::thread::spawn(move || {
            while let Ok(job) = receiver.recv() {
                // An aborted environment or callback panic cannot kill the
                // shared polling worker and strand unrelated promises.
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(job)));
            }
        });
        Self { sender }
    }
    fn submit(&self, job: Job) -> Result<(), Job> {
        self.sender.try_send(job).map_err(|error| match error {
            mpsc::TrySendError::Full(job) | mpsc::TrySendError::Disconnected(job) => job,
        })
    }
}

fn run_json(
    input: &[u8],
    run: fn(serde_json::Value) -> napi::Result<String>,
) -> napi::Result<String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::str::from_utf8(input)
            .map_err(|error| {
                napi::Error::from_reason(format!("options JSON must be UTF-8: {error}"))
            })
            .and_then(|input| {
                serde_json::from_str(input)
                    .map_err(|error| napi::Error::from_reason(error.to_string()))
            })
            .and_then(run)
    }))
    .unwrap_or_else(|_| Err(napi::Error::from_reason("progress polling panicked")))
}

// JS handles are confined to this thin promise boundary. CPU/JSON work still
// executes off the event loop; settling uses napi's threadsafe deferred queue.
#[cfg(not(coverage))]
mod promise {
    use super::Executor;
    use napi::bindgen_prelude::{Buffer, Object};
    use napi::{Env, JsDeferred, JsValue};
    use std::sync::OnceLock;
    type Resolver = Box<dyn FnOnce(Env) -> napi::Result<Buffer> + Send>;
    struct Job {
        input: Vec<u8>,
        run: fn(serde_json::Value) -> napi::Result<String>,
        deferred: JsDeferred<Buffer, Resolver>,
    }
    fn executor() -> &'static Executor<Job> {
        static WORKER: OnceLock<Executor<Job>> = OnceLock::new();
        WORKER.get_or_init(|| {
            Executor::start(32, |job: Job| {
                let output = super::run_json(&job.input, job.run);
                match output {
                    Ok(json) => job
                        .deferred
                        .resolve(Box::new(move |_| Ok(Buffer::from(json.into_bytes())))),
                    Err(error) => job.deferred.reject(error),
                }
            })
        })
    }
    pub(crate) fn json(
        env: Env,
        input: Buffer,
        run: fn(serde_json::Value) -> napi::Result<String>,
    ) -> napi::Result<Object<'static>> {
        // Buffer is backed by the owning JS isolate and can be mutated or
        // destroyed after this call. Only these small owned request bytes cross
        // to the independent worker; drop the JS reference on its owner thread.
        let owned_input = input.to_vec();
        drop(input);
        let (deferred, promise) = env.create_deferred::<Buffer, Resolver>()?;
        let promise = Object::from_raw(env.raw(), promise.raw());
        if let Err(job) = executor().submit(Job {
            input: owned_input,
            run,
            deferred,
        }) {
            job.deferred.reject(napi::Error::from_reason(
                "progress polling queue is full or stopped",
            ));
        }
        Ok(promise)
    }
}
#[cfg(not(coverage))]
pub(crate) use promise::json;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn executor_runs_independently_and_returns_overload_without_waiting() {
        let (started, ready) = mpsc::channel();
        let (release, resume) = mpsc::channel();
        let worker = Executor::start(1, move |job| {
            started.send(job).unwrap();
            resume.recv().unwrap();
        });
        worker.submit(1).unwrap();
        assert_eq!(ready.recv_timeout(Duration::from_secs(2)).unwrap(), 1);
        worker.submit(2).unwrap();
        assert_eq!(worker.submit(3), Err(3));
        release.send(()).unwrap();
        assert_eq!(ready.recv_timeout(Duration::from_secs(2)).unwrap(), 2);
        drop(worker);
        release.send(()).unwrap();
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;
    fn echo(value: serde_json::Value) -> napi::Result<String> {
        Ok(value.to_string())
    }
    #[test]
    fn owned_request_parsing_preserves_json_and_rejects_utf8_json_and_callback_errors() {
        assert_eq!(
            run_json(br#"{"id":"live"}"#, echo).unwrap(),
            r#"{"id":"live"}"#
        );
        assert!(run_json(&[0xff], echo)
            .unwrap_err()
            .reason
            .contains("UTF-8"));
        assert!(run_json(b"{", echo).is_err());
        fn reject(_: serde_json::Value) -> napi::Result<String> {
            Err(napi::Error::from_reason("invalid request"))
        }
        assert_eq!(
            run_json(b"{}", reject).unwrap_err().reason,
            "invalid request"
        );
        fn panic(_: serde_json::Value) -> napi::Result<String> {
            panic!("poll failed");
        }
        assert_eq!(
            run_json(b"{}", panic).unwrap_err().reason,
            "progress polling panicked"
        );
    }
    #[test]
    fn a_panicking_job_does_not_stop_other_polling_jobs() {
        let (sender, ready) = mpsc::channel();
        let worker = Executor::start(2, move |job| {
            if job == 0 {
                panic!("environment closed");
            }
            sender.send(job).unwrap();
        });
        worker.submit(0).unwrap();
        worker.submit(1).unwrap();
        assert_eq!(
            ready
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap(),
            1
        );
    }
}
