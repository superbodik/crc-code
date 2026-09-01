use serde::{Deserialize, Serialize};

/// Who made an edit.
///
/// Recorded on every transaction from the start, because collaborative undo
/// cannot be added later: it has to take back *your* last edit, not whatever
/// happens to sit on top of the stack. Retrofitting that means rewriting the
/// history, and by then every panel is reading from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AuthorId(pub u32);

impl AuthorId {
    /// The person at this keyboard. A buffer with no session has only this one.
    pub const LOCAL: AuthorId = AuthorId(0);

    /// An edit written by an agent, kept apart from the user's own so that
    /// undo after an agent run takes back the agent's work in one step.
    pub const AGENT: AuthorId = AuthorId(1);

    /// Ids from 2 up are handed out per collaborator by the session.
    pub const fn peer(index: u32) -> Self {
        AuthorId(index + 2)
    }

    pub const fn is_local(self) -> bool {
        self.0 == AuthorId::LOCAL.0
    }
}

impl Default for AuthorId {
    fn default() -> Self {
        AuthorId::LOCAL
    }
}
