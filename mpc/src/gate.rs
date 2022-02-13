#[derive(Clone)]
pub enum Gate<DataType: Clone> {
    Add {
        first: usize,
        second: usize,
        output: Option<DataType>
    },

    MulByConst {
        first: usize,
        second: DataType,
        output: Option<DataType>
    },

    Mul {
        first: usize,
        second: usize,
        output: Option<DataType>
    },

    Input {
        party: usize,
        circuit_id: usize,
        output: Option<DataType>
    }
}

impl<DataType: Clone> Gate<DataType> {

    pub fn new_input(party: usize, circuit_id: usize) -> Self {
        Gate::Input {
            party,
            circuit_id,
            output: None
        }
    }

    pub fn new_add(first: usize, second: usize) -> Self {
        Gate::Add {
            first, second,
            output: None
        }
    }

    pub fn new_mul_by_const(first: usize, second: DataType) -> Self {
        Gate::MulByConst {
            first, second,
            output: None
        }
    }

    pub fn new_mul(first: usize, second: usize) -> Self {
        Gate::Mul {
            first, second,
            output: None
        }
    }

    pub fn get_output(&self) -> DataType {
        match self {
            Gate::Input { party: _, circuit_id: _, output } => (*output).clone().unwrap(),
            Gate::Add { first: _, second: _, output } => (*output).clone().unwrap(),
            Gate::MulByConst {first: _, second: _, output } => (*output).clone().unwrap(),
            Gate::Mul { first: _, second: _, output } => (*output).clone().unwrap()
        }
    }

    pub fn set_output(&mut self, value: DataType) {
        match self {
            Gate::Input { party: _, circuit_id: _, output } => *output = Some(value),
            Gate::Add { first: _, second: _, output } => *output = Some(value),
            Gate::MulByConst {first: _, second: _, output } => *output = Some(value),
            Gate::Mul { first: _, second: _, output } => *output = Some(value)
        }
    }
}
