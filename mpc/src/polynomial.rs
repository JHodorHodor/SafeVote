use crate::field;

pub(crate) struct Polynomial<DataType> {
    coeffs: Vec<DataType>,
    field: field::Field<DataType>
}

impl<DataType: field::FieldElement + From<u16> + Clone> Polynomial<DataType> {

    pub(crate) fn random(c: DataType, degree: usize, mut field: field::Field<DataType>) -> Polynomial<DataType> {
        Polynomial {
            coeffs: std::iter::once(c).chain((1..degree + 1).map(|_| field.random())).collect(),
            field
        }
    }

    pub(crate) fn eval(&self, x: DataType) -> DataType {
        let mut result = self.field.zero();

        for coeff in self.coeffs.iter().rev() {
            result = self.field.mul(result, x.clone());
            result = self.field.add(result, (*coeff).clone());
        }

        result
    }

    pub(crate) fn lagrange(knots: impl Iterator<Item = DataType>, i: DataType, field: &field::Field<DataType>) -> DataType {
        knots
            .map(|knot| {
                if knot == i {
                    field.one()
                } else {
                    field.mul(knot.clone(), field.inv(field.sub(knot, i.clone())))
                }
            })
            .fold(field.one(), |a, b| field.mul(a, b))
    }

    pub(crate) fn interpolate(shares: Vec<DataType>, field: &field::Field<DataType>, n_parties: u16) -> DataType {
        (0..n_parties)
                .map(|party| (shares[party as usize].clone(),
                    Polynomial::lagrange(
                        (0..n_parties).map(|i| DataType::from(i + 1)),
                        DataType::from(party + 1),
                        &field
                    )))
                .map(|(share, lagr)| field.mul(share, lagr))
                .fold(field.zero(), |a, b| field.add(a, b))
    }
}