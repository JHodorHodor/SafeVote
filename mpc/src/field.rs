use std::ops::{Add, Mul, Rem};
use rand::rngs::ThreadRng;
use rand::Rng;
use num::Zero;

pub trait FieldElement: Add<Output = Self> +
                        Mul<Output = Self> +
                        Rem<Output = Self> +
                        PartialOrd +
                        Zero +
                        Clone +
                        rand::distributions::uniform::SampleUniform
where Self: Sized {

}

impl<T> FieldElement for T
where T: Add<Output = T> +
         Mul<Output = T> +
         Rem<Output = T> +
         PartialOrd +
         Zero +
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

    pub fn zero(&self) -> DataType {
        DataType::zero()
    }

    pub fn add(&self, a: DataType, b: DataType) -> DataType {
        self.normalize(a + b)
    }

    pub fn mul(&self, a: DataType, b: DataType) -> DataType {
        self.normalize(a * b)
    }

    pub fn random(&mut self) -> DataType {
        self.rand.gen_range(DataType::zero()..self.order.clone())
    }

    fn normalize(&self, a: DataType) -> DataType {
        // TODO: negative?
        a % self.order.clone()
    }
}