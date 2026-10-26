impl VitestSetupField {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SetupFiles => "setupFiles",
            Self::GlobalSetup => "globalSetup",
        }
    }
}
