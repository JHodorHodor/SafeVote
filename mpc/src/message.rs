use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct Message<DataType> {
    from: usize,
    to: usize,
    gate: usize,
    share: DataType
}

impl<DataType: Clone> Message<DataType> {

    pub fn new(from: usize, to: usize, gate: usize, share: DataType) -> Self {
        Message {
            from, to, gate, share
        }
    }

    pub(crate) fn get_from(&self) -> usize {
        self.from
    }

    pub(crate) fn get_to(&self) -> usize {
        self.to
    }

    pub(crate) fn get_gate(&self) -> usize {
        self.gate
    }

    pub(crate) fn get_share(&self) -> DataType {
        self.share.clone()
    }
}

impl<DataType> PartialEq for Message<DataType> {
    fn eq(&self, other: &Self) -> bool {
        return
            self.from.eq(&other.from) &&
            self.to.eq(&other.to) &&
            self.gate.eq(&other.gate);
    }
}

impl<DataType> Eq for Message<DataType> { }

impl<DataType> Hash for Message<DataType> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        self.gate.hash(state);
    }
}