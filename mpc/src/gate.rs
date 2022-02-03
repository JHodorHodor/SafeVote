#[derive(Clone)]
pub enum Gate<DataType: Clone> {
    Add {
        first: Box<Gate<DataType>>,
        second: Box<Gate<DataType>>,
        output: Option<DataType>
    },

    MulByConst {
        first: Box<Gate<DataType>>,
        second: DataType,
        output: Option<DataType>
    },

    Mul {
        first: Box<Gate<DataType>>,
        second: Box<Gate<DataType>>,
        output: Option<DataType>
    },

    Input {
        party: usize,
        output: Option<DataType>
    }
}

impl<DataType: Clone> Gate<DataType> {

    pub fn new_input(party: usize) -> Self {
        Gate::Input {
            party,
            output: None
        }
    }

    pub fn new_add(first: Box<Gate<DataType>>, second: Box<Gate<DataType>>) -> Self {
        Gate::Add {
            first, second,
            output: None
        }
    }

    pub fn new_mul_by_const(first: Box<Gate<DataType>>, second: DataType) -> Self {
        Gate::MulByConst {
            first, second,
            output: None
        }
    }

    pub fn new_mul(first: Box<Gate<DataType>>, second: Box<Gate<DataType>>) -> Self {
        Gate::Mul {
            first, second,
            output: None
        }
    }

    pub fn get_output(&self) -> DataType {
        match self {
            Gate::Input { party: _, output } => (*output).clone().unwrap(),
            Gate::Add { first: _, second: _, output } => (*output).clone().unwrap(),
            Gate::MulByConst {first: _, second: _, output } => (*output).clone().unwrap(),
            Gate::Mul { first: _, second: _, output } => (*output).clone().unwrap()
        }
    }
}
