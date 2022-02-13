use std::ops::{Add, Sub, Mul, Rem};
use rand::rngs::ThreadRng;
use rand::Rng;
use num::{Zero, One};

pub trait FieldElement: Add<Output = Self> +
                        Sub<Output = Self> +
                        Mul<Output = Self> +
                        Rem<Output = Self> +
                        PartialOrd +
                        Zero +
                        One +
                        Clone +
                        rand::distributions::uniform::SampleUniform
where Self: Sized {}

impl<T> FieldElement for T
where T: Add<Output = T> +
         Sub<Output = Self> +
         Mul<Output = T> +
         Rem<Output = T> +
         PartialOrd +
         Zero +
         One +
         Clone +
         rand::distributions::uniform::SampleUniform {}

#[derive(Clone)]
pub struct Field<DataType> {
    order: DataType,
    rand: ThreadRng,
}

impl<DataType: FieldElement> Field<DataType> {

    pub fn new(order: DataType) -> Self {
        Field { order, rand: rand::thread_rng() }
    }

    pub(crate) fn zero(&self) -> DataType {
        DataType::zero()
    }

    pub(crate) fn one(&self) -> DataType {
        DataType::one()
    }

    pub fn add(&self, a: DataType, b: DataType) -> DataType {
        self.normalize(a + b)
    }

    pub fn sub(&self, a: DataType, b: DataType) -> DataType {
        if a >= b {
            self.normalize(a - b)
        } else {
            self.order.clone() - self.normalize(b - a)
        }
    }

    pub fn mul(&self, a: DataType, b: DataType) -> DataType {
        self.normalize(a * b)
    }

    pub fn inv(&self, a: DataType) -> DataType {
        self.pow(a, self.order.clone() - self.one() - self.one())
    }

    pub fn random(&mut self) -> DataType {
        self.rand.gen_range(DataType::zero()..self.order.clone())
    }

    fn normalize(&self, a: DataType) -> DataType {
        // NOTE: DataType is assumed to be nonnegative
        a % self.order.clone()
    }

    fn pow(&self, a: DataType, b: DataType) -> DataType {
        let mut result = self.one();
        
        let mut counter = self.zero();
        while counter != b {
            result  = self.mul(result, a.clone());
            counter = counter + self.one();
        }

        result
    }
}


#[cfg(test)]
mod tests {
    use super::Field;

    #[test]
    fn test_zero() {
        let field = Field::new(13u8);
        assert_eq!(field.zero(), 0u8);
    }

    #[test]
    fn test_one() {
        let field = Field::new(13u8);
        assert_eq!(field.one(), 1u8);
    }

    #[test]
    fn test_add() {
        let field = Field::new(13u8);
        assert_eq!(field.add(7u8, 8u8), 2u8);
    }

    #[test]
    fn test_sub() {
        let field = Field::new(13u8);
        assert_eq!(field.sub(7u8, 8u8), 12u8);
    }

    #[test]
    fn test_mul() {
        let field = Field::new(13u8);
        assert_eq!(field.mul(7u8, 8u8), 4u8);
    }

    #[test]
    fn test_inv() {
        let field = Field::new(13u8);
        assert_eq!(field.inv(7u8), 2u8);
    }

    #[test]
    fn test_random() {
        let mut field = Field::new(13u8);
        assert!(field.random() < 13u8);
    }
}