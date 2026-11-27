use super::Cursor;

impl Cursor<'_> {
    pub(in super::super) fn role(&mut self) -> bool {
        ["CURRENT_ROLE", "CURRENT_USER", "SESSION_USER"]
            .iter()
            .any(|word| self.word(word))
            || self.identifier()
    }
    pub(in super::super) fn mapping_role(&mut self) -> bool {
        [
            "CURRENT_ROLE",
            "CURRENT_USER",
            "SESSION_USER",
            "USER",
            "PUBLIC",
        ]
        .iter()
        .any(|word| self.word(word))
            || self.identifier()
    }
    pub(in super::super) fn owned_role(&mut self) -> bool {
        if self.word("PUBLIC") {
            return false;
        }
        ["CURRENT_ROLE", "CURRENT_USER", "SESSION_USER"]
            .iter()
            .any(|word| self.word(word))
            || self.identifier()
    }
}
