use super::data;
use pinot::FontRef;

pub struct Cache {
    serial: u64,
    entries: Vec<Entry>,
    max_entries: usize,
}

struct Entry {
    id: u64,
    cached: data::Cached,
    serial: u64,
}

impl Cache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            serial: 0,
            entries: vec![],
            max_entries,
        }
    }

    pub fn get(&mut self, font: &FontRef, id: u64) -> data::Cached {
        let (found, index) = self.find(id);
        if found {
            let entry = &mut self.entries[index];
            entry.serial = self.serial;
            entry.cached
        } else {
            self.serial += 1;
            let cached = data::Cached::new(font);
            if index == self.entries.len() {
                self.entries.push(Entry {
                    id,
                    cached,
                    serial: self.serial,
                });
            } else {
                let entry = &mut self.entries[index];
                entry.id = id;
                entry.cached = cached;
                entry.serial = self.serial;
            }
            cached
        }
    }

    fn find(&self, id: u64) -> (bool, usize) {
        let mut lowest_index = 0;
        let mut lowest_serial = self.serial;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.id == id {
                return (true, i);
            }
            if entry.serial < lowest_serial {
                lowest_serial = entry.serial;
                lowest_index = i;
            }
        }
        if self.entries.len() < self.max_entries {
            (false, self.entries.len())
        } else {
            (false, lowest_index)
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new(16)
    }
}
