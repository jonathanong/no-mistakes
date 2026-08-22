use std::path::Path;

/// Discovery-time file classification for one lexical inventory path.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[doc(hidden)]
pub struct FileClassification {
    lexical_file: bool,
    lexical_symlink: bool,
    target_file: bool,
}

impl FileClassification {
    pub(crate) const TRACKED_REGULAR: Self = Self {
        lexical_file: true,
        lexical_symlink: false,
        target_file: true,
    };

    pub(crate) fn from_file_type(path: &Path, file_type: std::fs::FileType) -> Self {
        let lexical_file = file_type.is_file();
        let lexical_symlink = file_type.is_symlink();
        Self {
            lexical_file,
            lexical_symlink,
            target_file: lexical_file || (lexical_symlink && path.is_file()),
        }
    }

    #[doc(hidden)]
    pub fn is_lexical_file(self) -> bool {
        self.lexical_file
    }

    #[doc(hidden)]
    pub fn is_lexical_symlink(self) -> bool {
        self.lexical_symlink
    }

    #[doc(hidden)]
    pub fn target_is_file(self) -> bool {
        self.target_file
    }
}
