use crate::author::AuthorId;
use crate::edit::{Change, rebase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub author: AuthorId,
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

    pub fn inverted(&self) -> Transaction {
        Transaction {
            author: self.author,
            changes: self.changes.iter().rev().map(Change::inverted).collect(),
        }
    }

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

#[derive(Debug)]
pub struct History {
    done: Vec<Transaction>,
    undone: Vec<Transaction>,
    pending: Transaction,
    limit: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(512)
    }
}

impl History {
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

    pub fn can_undo_by(&self, author: AuthorId) -> bool {
        (!self.pending.is_empty() && self.pending.author == author)
            || self.done.iter().any(|t| t.author == author)
    }

    pub fn push(&mut self, change: Change, author: AuthorId) {
        self.undone.clear();
        if (!self.pending.is_empty() && self.pending.author != author)
            || !self.continues_pending(&change)
        {
            self.commit();
        }
        self.pending.author = author;

        let ends_group = change.inserted.contains('\n');
        self.pending.changes.push(change);
        if ends_group {
            self.commit();
        }
    }

    pub fn push_transaction(&mut self, transaction: Transaction) {
        if transaction.is_empty() {
            return;
        }
        self.undone.clear();
        self.commit();
        self.done.push(transaction);
        self.trim();
    }

    pub fn commit(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let transaction = std::mem::take(&mut self.pending);
        self.done.push(transaction);
        self.trim();
    }

    pub fn undo(&mut self) -> Option<Transaction> {
        self.commit();
        let transaction = self.done.pop()?;
        let inverse = transaction.inverted();
        self.undone.push(transaction);
        Some(inverse)
    }

    pub fn undo_by(&mut self, author: AuthorId) -> Option<Transaction> {
        self.commit();
        let index = self.done.iter().rposition(|t| t.author == author)?;

        let mut inverse = self.done[index].inverted();
        for later in &self.done[index + 1..] {
            inverse = inverse.rebased(later)?;
        }

        self.done.remove(index);
        self.undone.push(inverse.inverted());
        Some(inverse)
    }

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
