use {TextRange, TextUnit};

pub struct Edit {
    atoms: Vec<AtomEdit>,
}

pub struct AtomEdit {
    delete: TextRange,
    insert: String,
}

pub struct EditBuilder {
    atoms: Vec<AtomEdit>
}

impl EditBuilder {
    pub fn new() -> EditBuilder {
        EditBuilder { atoms: Vec::new() }
    }

    pub fn replace(&mut self, range: TextRange, replacement: String) {
        let range = self.translate(range);
        self.atoms.push(AtomEdit { delete: range, insert: replacement })
    }

    pub fn delete(&mut self, range: TextRange) {
        self.replace(range, String::new());
    }

    pub fn insert(&mut self, offset: TextUnit, text: String) {
        self.replace(TextRange::offset_len(offset, 0.into()), text)
    }

    pub fn finish(self) -> Edit {
        Edit { atoms: self.atoms }
    }

    fn translate(&self, range: TextRange) -> TextRange {
        let mut range = range;
        for atom in self.atoms.iter() {
            range = atom.apply_to_range(range)
                .expect("conflicting edits");
        }
        range
    }
}


impl AtomEdit {
    fn apply_to_position(&self, pos: TextUnit) -> Option<TextUnit> {
        if self.delete.start() >= pos {
            return Some(pos);
        }
        if self.delete.end() > pos {
            return None;
        }
        Some(pos - self.delete.len() + TextUnit::of_str(&self.insert))
    }

    fn apply_to_range(&self, range: TextRange) -> Option<TextRange> {
        Some(TextRange::from_to(
            self.apply_to_position(range.start())?,
            self.apply_to_position(range.end())?,
        ))
    }
}

