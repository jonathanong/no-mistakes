use super::PathOrigin;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CapturedEnv {
    pub(crate) name: String,
    pub(crate) offset: u32,
}

impl PathOrigin {
    pub(crate) fn environment(&self) -> Option<(&str, Option<u32>)> {
        self.env.as_deref().map(|name| (name, None)).or_else(|| {
            self.captured_env
                .as_deref()
                .map(|capture| (capture.name.as_str(), Some(capture.offset)))
        })
    }
}
