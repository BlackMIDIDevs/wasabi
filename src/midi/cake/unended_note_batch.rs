use std::collections::{BTreeMap, VecDeque};

pub struct RemovedValue<T> {
    pub value: T,
    pub is_last: bool,
}

pub struct UnendedNotes<K: Ord, T> {
    id_counter: u32,
    notes: BTreeMap<u32, T>,
    ids: BTreeMap<K, VecDeque<u32>>,
}

impl<K: Ord, T> UnendedNotes<K, T> {
    pub fn new() -> Self {
        UnendedNotes {
            id_counter: 0,
            notes: BTreeMap::new(),
            ids: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.notes.len()
    }

    pub fn top_mut(&mut self) -> Option<&mut T> {
        let key = *self.notes.last_entry()?.key();
        self.notes.get_mut(&key)
    }

    pub fn get_note_for(&mut self, key: K) -> Option<RemovedValue<T>> {
        let ids = self.ids.get_mut(&key)?;
        let id = ids.pop_front()?;
        let last_key = *self.notes.last_entry()?.key();

        let note = self.notes.remove(&id)?;
        Some(RemovedValue {
            value: note,
            is_last: id == last_key,
        })
    }

    pub fn push_note(&mut self, key: K, note: T) -> u32 {
        let id = self.id_counter;
        self.id_counter += 1;

        let ids = self.ids.entry(key).or_default();
        ids.push_back(id);

        self.notes.insert(id, note);

        id
    }

    pub fn drain_all(&mut self) -> impl '_ + Iterator<Item = T> {
        let notes = std::mem::take(&mut self.notes);
        self.ids = BTreeMap::new();

        notes.into_values()
    }
}
