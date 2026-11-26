use super::PathOrigin;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapturedEnv {
    pub name: String,
    pub offset: u32,
}

impl PathOrigin {
    pub fn environment(&self) -> Option<(&str, Option<u32>)> {
        self.env.as_deref().map(|name| (name, None)).or_else(|| {
            self.captured_env
                .as_deref()
                .map(|capture| (capture.name.as_str(), Some(capture.offset)))
        })
    }
}
