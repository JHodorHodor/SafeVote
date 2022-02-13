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
where Self: Sized {

}

impl<T> FieldElement for T
where T: Add<Output = T> +
         Sub<Output = Self> +
         Mul<Output = T> +
         Rem<Output = T> +
         PartialOrd +
         Zero +
         One +
         Clone +
         rand::distributions::uniform::SampleUniform {
             
         }

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
        // TODO: negative?
        a % self.order.clone()
    }

    fn pow(&self, a: DataType, b: DataType) -> DataType {
        let mut result = self.one();
        
        // TODO: binpow
        /* while b != self.zero() {
            if b & 1 != self.zero() {
                result = self.mul(result, a);
            }
            b >>= 1;
            a = self.mul(a, a);
        } */

        let mut counter = self.zero();
        while counter != b {
            result  = self.mul(result, a.clone());
            counter = counter + self.one();
        }

        result
    }
}