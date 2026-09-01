use crate::author::AuthorId;
use crate::edit::{Change, rebase};

/// A group of changes that undo and redo as one step, and who made them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub author: AuthorId,
    /// In the order they were applied.
    pub changes: Vec<Change>,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            author: AuthorId::LOCAL,
            changes: Vec::new(),
        }
    }
}

impl Transaction {
    pub fn new(author: AuthorId) -> Self {
        Self {
            author,
            changes: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// The transaction that undoes this one: every change inverted, and
    /// replayed back to front so each range is still valid when it is applied.
    pub fn inverted(&self) -> Transaction {
        Transaction {
            author: self.author,
            changes: self.changes.iter().rev().map(Change::inverted).collect(),
        }
    }

    /// Move this transaction into the coordinates that `over` left behind.
    ///
    /// `None` if the two touch the same text — see [`rebase`].
    pub fn rebased(&self, over: &Transaction) -> Option<Transaction> {
        let mut changes = Vec::with_capacity(self.changes.len());
        for change in &self.changes {
            let mut range = change.range.clone();
            for later in &over.changes {
                range = rebase(&range, later)?;
            }
            changes.push(Change {
                range,
                removed: change.removed.clone(),
                inserted: change.inserted.clone(),
            });
        }
        Some(Transaction {
            author: self.author,
            changes,
        })
    }
}

/// Undo and redo stacks.
///
/// Consecutive typing coalesces into one transaction, so undo steps back over a
/// word rather than one character at a time. Anything that is not a
/// continuation of the last edit — a jump elsewhere, a deletion, a different
/// author, an explicit [`commit`](History::commit) — closes the group.
#[derive(Debug)]
pub struct History {
    done: Vec<Transaction>,
    undone: Vec<Transaction>,
    /// The group still being typed into.
    pending: Transaction,
    limit: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(512)
    }
}

impl History {
    /// `limit` caps the undo stack so a long session cannot grow without bound.
    pub fn new(limit: usize) -> Self {
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            pending: Transaction::default(),
            limit,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.pending.is_empty() || !self.done.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// Whether `author` has anything left to take back.
    pub fn can_undo_by(&self, author: AuthorId) -> bool {
        (!self.pending.is_empty() && self.pending.author == author)
            || self.done.iter().any(|t| t.author == author)
    }

    /// Record a change. A new edit always invalidates the redo stack.
    pub fn push(&mut self, change: Change, author: AuthorId) {
        self.undone.clear();
        if (!self.pending.is_empty() && self.pending.author != author)
            || !self.continues_pending(&change)
        {
            self.commit();
        }
        self.pending.author = author;

        // A newline ends the group it belongs to, so undo takes back the line
        // that was just typed and leaves the break above it.
        let ends_group = change.inserted.contains('\n');
        self.pending.changes.push(change);
        if ends_group {
            self.commit();
        }
    }

    /// Record a group of changes as one indivisible undo step — a multi-cursor
    /// edit, a formatter pass, a diff applied by an agent.
    pub fn push_transaction(&mut self, transaction: Transaction) {
        if transaction.is_empty() {
            return;
        }
        self.undone.clear();
        self.commit();
        self.done.push(transaction);
        self.trim();
    }

    /// Close the current group, so the next edit starts a fresh undo step.
    ///
    /// Call it when the cursor jumps, on save, or after an idle pause.
    pub fn commit(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let transaction = std::mem::take(&mut self.pending);
        self.done.push(transaction);
        self.trim();
    }

    /// The transaction to apply in order to step back, if there is one.
    pub fn undo(&mut self) -> Option<Transaction> {
        self.commit();
        let transaction = self.done.pop()?;
        let inverse = transaction.inverted();
        self.undone.push(transaction);
        Some(inverse)
    }

    /// Step back over the newest edit made by `author`, whoever has typed
    /// since.
    ///
    /// The inverse is moved past every transaction applied after it, so
    /// undoing your own edit while a collaborator works further down the file
    /// does the right thing. `None` if `author` has nothing to undo, or if a
    /// later edit touched the same text — that is a conflict, and guessing at
    /// it would mean quietly discarding someone else's work.
    pub fn undo_by(&mut self, author: AuthorId) -> Option<Transaction> {
        self.commit();
        let index = self.done.iter().rposition(|t| t.author == author)?;

        let mut inverse = self.done[index].inverted();
        for later in &self.done[index + 1..] {
            inverse = inverse.rebased(later)?;
        }

        self.done.remove(index);
        // Store the redo in the coordinates the undo is about to create, not
        // the ones it was originally written in.
        self.undone.push(inverse.inverted());
        Some(inverse)
    }

    /// The transaction to apply in order to step forward, if there is one.
    pub fn redo(&mut self) -> Option<Transaction> {
        let transaction = self.undone.pop()?;
        self.done.push(transaction.clone());
        Some(transaction)
    }

    fn trim(&mut self) {
        if self.done.len() > self.limit {
            self.done.remove(0);
        }
    }

    /// Typing continues a group only when it is a plain insertion picking up
    /// exactly where the last one left off. A deletion, a replacement, or a
    /// jump elsewhere all start a new step.
    fn continues_pending(&self, change: &Change) -> bool {
        let Some(last) = self.pending.changes.last() else {
            return true;
        };
        change.removed.is_empty()
            && last.removed.is_empty()
            && !change.inserted.is_empty()
            && change.range.start == last.applied_range().end
    }
}
