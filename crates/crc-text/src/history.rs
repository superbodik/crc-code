use crate::edit::Change;

/// A group of changes that undo and redo as one step.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Transaction {
    /// In the order they were applied.
    pub changes: Vec<Change>,
}

impl Transaction {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// The transaction that undoes this one: every change inverted, and
    /// replayed back to front so each range is still valid when it is applied.
    pub fn inverted(&self) -> Transaction {
        Transaction {
            changes: self.changes.iter().rev().map(Change::inverted).collect(),
        }
    }
}

/// Undo and redo stacks.
///
/// Consecutive typing coalesces into one transaction, so undo steps back over a
/// word rather than one character at a time. Anything that is not a
/// continuation of the last edit — a jump elsewhere, a deletion, an explicit
/// [`commit`](History::commit) — closes the group.
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

    /// Record a change. A new edit always invalidates the redo stack.
    pub fn push(&mut self, change: Change) {
        self.undone.clear();
        if !self.continues_pending(&change) {
            self.commit();
        }
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
        if self.done.len() > self.limit {
            self.done.remove(0);
        }
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
        if self.done.len() > self.limit {
            self.done.remove(0);
        }
    }

    /// The transaction to apply in order to step back, if there is one.
    pub fn undo(&mut self) -> Option<Transaction> {
        self.commit();
        let transaction = self.done.pop()?;
        let inverse = transaction.inverted();
        self.undone.push(transaction);
        Some(inverse)
    }

    /// The transaction to apply in order to step forward, if there is one.
    pub fn redo(&mut self) -> Option<Transaction> {
        let transaction = self.undone.pop()?;
        self.done.push(transaction.clone());
        Some(transaction)
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
