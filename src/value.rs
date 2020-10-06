use std::ops::Neg;

#[derive(Copy, Clone)]
pub struct Value(pub f64);

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        Value(-self.0)
    }
}

impl Value {
    pub fn print(&self) {
        print!("{}", self.0);
    }
}
