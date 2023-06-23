use ndarray::{ArrayBase, Data, Dimension};

pub trait AlmostEq<Rhs> {
    fn almost_eq(&self, rhs: &Rhs, accuracy: f64) -> bool;
}

impl AlmostEq<f64> for f64 {
    fn almost_eq(&self, rhs: &f64, accuracy: f64) -> bool {
        (self - rhs).abs() < accuracy
    }
}

impl<S, S2, D> AlmostEq<ArrayBase<S2, D>> for ArrayBase<S, D>
where
    S: Data<Elem = f64>,
    S2: Data<Elem = f64>,
    D: Dimension,
{
    fn almost_eq(&self, rhs: &ArrayBase<S2, D>, accuracy: f64) -> bool {
        (self - rhs).iter().all(|diff| diff.abs() < accuracy)
    }
}
