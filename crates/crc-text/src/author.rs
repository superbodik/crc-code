use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AuthorId(pub u32);

impl AuthorId {
    pub const LOCAL: AuthorId = AuthorId(0);

    pub const AGENT: AuthorId = AuthorId(1);

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
