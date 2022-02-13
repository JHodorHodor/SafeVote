use crate::field;

pub(crate) struct Polynomial<DataType> {
    coeffs: Vec<DataType>,
    field: field::Field<DataType>
}

impl<DataType: field::FieldElement + Clone> Polynomial<DataType> {

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

    pub(crate) fn interpolate<F>(shares: Vec<DataType>, 
                                field: &field::Field<DataType>, 
                                n_parties: usize, 
                                convert: F) -> DataType 
    where 
        F: Fn(usize)->DataType
    {
        (0..n_parties)
                .map(|party| (shares[party].clone(),
                    Polynomial::lagrange(
                        (0..n_parties).map(|i| convert(i + 1)),
                        convert(party + 1),
                        &field
                    )))
                .map(|(share, lagr)| field.mul(share, lagr))
                .fold(field.zero(), |a, b| field.add(a, b))
    }
}

#[cfg(test)]
mod tests {
    use super::Polynomial;
    use super::field::Field;

    #[test]
    fn test_random() {
        let poly = Polynomial::random(2u8, 5, Field::new(13u8));
        assert_eq!(poly.coeffs.len(), 6);
        assert_eq!(poly.coeffs[0], 2u8);
    }

    #[test]
    fn test_eval() {
        let poly = Polynomial::random(2u8, 2, Field::new(13u8));
        let c0 = 2u8;
        let c1 = poly.coeffs[1];
        let c2 = poly.coeffs[2];

        assert_eq!(poly.eval(0), c0);
        assert_eq!(poly.eval(1), (c0 + c1 + c2) % 13u8);
        assert_eq!(poly.eval(2), (c0 + 2 * c1 + 4 * c2) % 13u8);
    }

    #[test]
    fn test_lagrange() {
        let field = Field::new(13u8);
        assert_eq!(Polynomial::lagrange(1..4, 2, &field), 10u8);
    }

    #[test]
    fn test_interpolate() {
        let field = Field::new(13u16);
        let poly = Polynomial::random(2u16, 2, field.clone());
        assert_eq!(Polynomial::interpolate(
            (1..4).map(|i| poly.eval(i)).collect(),
            &field, 3, |x| x as u16
        ), 2u16);
    }
}