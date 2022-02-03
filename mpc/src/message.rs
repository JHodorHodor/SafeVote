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

    pub fn get_share(&self) -> DataType {
        self.share.clone()
    }
}