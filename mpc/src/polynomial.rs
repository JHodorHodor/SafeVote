use crate::field;

pub struct Polynomial<DataType> {
    coeffs: Vec<DataType>,
    field: field::Field<DataType>
}

impl<DataType: field::FieldElement + Clone> Polynomial<DataType> {

    pub fn random(c: DataType, degree: usize, mut field: field::Field<DataType>) -> Polynomial<DataType> {
        Polynomial {
            coeffs: std::iter::once(c).chain((1..degree).map(|_| field.random())).collect(),
            field
        }
    }

    pub fn eval(&self, x: DataType) -> DataType {
        let mut result = self.field.zero();

        for coeff in self.coeffs.iter().rev() {
            result = self.field.mul(result, x.clone());
            result = self.field.add(result, (*coeff).clone());
        }

        result
    }

    pub fn lagrange(&self, knots: impl Iterator<Item = DataType>, i: DataType) -> DataType {
        knots
            .map(|knot| {
                if knot == i {
                    self.field.one()
                } else {
                    self.field.mul(knot.clone(), self.field.inv(self.field.sub(knot, i.clone())))
                }
            })
            .fold(self.field.one(), |a, b| self.field.mul(a, b))
    }
}