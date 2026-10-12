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
                run(job);
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
        input: Buffer,
        run: fn(serde_json::Value) -> napi::Result<String>,
        deferred: JsDeferred<Buffer, Resolver>,
    }
    fn executor() -> &'static Executor<Job> {
        static WORKER: OnceLock<Executor<Job>> = OnceLock::new();
        WORKER.get_or_init(|| {
            Executor::start(32, |job: Job| {
                let output = std::str::from_utf8(&job.input)
                    .map_err(|error| {
                        napi::Error::from_reason(format!("options JSON must be UTF-8: {error}"))
                    })
                    .and_then(|input| {
                        serde_json::from_str(input)
                            .map_err(|error| napi::Error::from_reason(error.to_string()))
                    })
                    .and_then(job.run);
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
        let (deferred, promise) = env.create_deferred::<Buffer, Resolver>()?;
        let promise = Object::from_raw(env.raw(), promise.raw());
        if let Err(job) = executor().submit(Job {
            input,
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
