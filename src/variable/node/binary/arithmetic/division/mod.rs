#[cfg(test)]
use super::{assert_almost_equals, new_backward_input, new_input, new_tensor};
use super::{
    cobroadcasted_zeros, expect_tensor, expect_tensor_mut, push_gradient, reduce, Backward,
    BroadTensor, Broadcasted, Cache, Data, Forward, Gradient, Overwrite, Tensor,
};
use ndarray::{DimMax, Dimension, Zip};
use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    fmt::{Debug, Display},
    rc::Rc,
};

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Division ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub struct Division<Lhs: ?Sized, Rhs: ?Sized>
where
    Lhs: Data,
    Rhs: Data,
    Lhs::Dim: Dimension + DimMax<Rhs::Dim>,
{
    left: Rc<Lhs>,
    right: Rc<Rhs>,
    data: RefCell<BroadTensor<Lhs::Dim, Rhs::Dim>>,
    computed: Cell<bool>,
}

impl<Lhs: ?Sized, Rhs: ?Sized> Division<Lhs, Rhs>
where
    Lhs: Data,
    Rhs: Data,
    Lhs::Dim: Dimension + DimMax<Rhs::Dim>,
{
    pub fn new(left: Rc<Lhs>, right: Rc<Rhs>) -> Self {
        let data = RefCell::new(cobroadcasted_zeros(&left.data(), &right.data()));

        Self {
            left,
            right,
            data,
            computed: Cell::new(false),
        }
    }
}

impl<Lhs: ?Sized, Rhs: ?Sized> Data for Division<Lhs, Rhs>
where
    Lhs: Data,
    Rhs: Data,
    Lhs::Dim: Dimension + DimMax<Rhs::Dim>,
{
    type Dim = Broadcasted<Lhs::Dim, Rhs::Dim>;

    fn data(&self) -> Ref<Tensor<Self::Dim>> {
        self.data.borrow()
    }

    fn data_mut(&self) -> RefMut<Tensor<Self::Dim>> {
        self.data.borrow_mut()
    }
}

impl<Lhs: ?Sized, Rhs: ?Sized> Cache for Division<Lhs, Rhs>
where
    Lhs: Data,
    Rhs: Data,
    Lhs::Dim: Dimension + DimMax<Rhs::Dim>,
{
    fn was_computed(&self) -> bool {
        self.computed.get()
    }

    fn reset_computation(&self) {
        self.computed.set(false);
    }
}

impl<Lhs: ?Sized, Rhs: ?Sized> Forward for Division<Lhs, Rhs>
where
    Lhs: Data,
    Rhs: Data,
    Lhs::Dim: Dimension + DimMax<Rhs::Dim>,
{
    fn forward(&self) {
        if self.was_computed() {
            return;
        }

        self.computed.set(true);
        Zip::from(&mut *self.data.borrow_mut())
            .and_broadcast(&*self.left.data())
            .and_broadcast(&*self.right.data())
            .for_each(|v, l, r| *v = l / r);
    }
}

impl<Lhs: ?Sized, Rhs: ?Sized> Debug for Division<Lhs, Rhs>
where
    Lhs: Data,
    Rhs: Data,
    Lhs::Dim: Dimension + DimMax<Rhs::Dim>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Division")
            .field("data", &self.data.borrow())
            .field("computed", &self.computed.get())
            .finish()
    }
}

impl<Lhs: ?Sized, Rhs: ?Sized> Display for Division<Lhs, Rhs>
where
    Lhs: Data,
    Rhs: Data,
    Lhs::Dim: Dimension + DimMax<Rhs::Dim>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data.borrow())
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ DivisionBackward ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub struct DivisionBackward<LhsD: ?Sized, LhsG: ?Sized, RhsD: ?Sized, RhsG: ?Sized>
where
    LhsD: Data,
    RhsD: Data,
    LhsG: Gradient,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsD::Dim>,
    LhsG::Dim: Dimension + DimMax<RhsG::Dim>,
{
    gradient: RefCell<Option<BroadTensor<LhsG::Dim, RhsG::Dim>>>,
    shape: Broadcasted<LhsG::Dim, RhsG::Dim>,
    overwrite: Cell<bool>,
    buffer: RefCell<Option<BroadTensor<LhsG::Dim, RhsG::Dim>>>,
    left_data: Rc<LhsD>,
    left_grad: Rc<LhsG>,
    right_data: Rc<RhsD>,
    right_grad: Rc<RhsG>,
}

impl<LhsD: ?Sized, LhsG: ?Sized, RhsD: ?Sized, RhsG: ?Sized>
    DivisionBackward<LhsD, LhsG, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    LhsG: Gradient,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsD::Dim>,
    LhsG::Dim: Dimension + DimMax<RhsG::Dim>,
{
    pub fn new(
        left_data: Rc<LhsD>,
        left_grad: Rc<LhsG>,
        right_data: Rc<RhsD>,
        right_grad: Rc<RhsG>,
    ) -> Self {
        let gradient = cobroadcasted_zeros(&left_grad.gradient(), &right_grad.gradient());
        let shape = gradient.raw_dim();

        Self {
            gradient: RefCell::new(Some(gradient)),
            shape: shape.clone(),
            overwrite: Cell::new(true),
            buffer: RefCell::new(Some(Tensor::zeros(shape))),
            left_data,
            left_grad,
            right_data,
            right_grad,
        }
    }
}

impl<LhsD: ?Sized, LhsG: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Gradient
    for DivisionBackward<LhsD, LhsG, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    LhsG: Gradient,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsD::Dim>,
    LhsG::Dim: Dimension + DimMax<RhsG::Dim>,
{
    type Dim = Broadcasted<LhsG::Dim, RhsG::Dim>;

    fn gradient(&self) -> Ref<Tensor<Self::Dim>> {
        expect_tensor(&self.gradient)
    }

    fn gradient_mut(&self) -> RefMut<Tensor<Self::Dim>> {
        expect_tensor_mut(&self.gradient)
    }
}

impl<LhsD: ?Sized, LhsG: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Overwrite
    for DivisionBackward<LhsD, LhsG, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    LhsG: Gradient,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsD::Dim>,
    LhsG::Dim: Dimension + DimMax<RhsG::Dim>,
{
    fn can_overwrite(&self) -> bool {
        self.overwrite.get()
    }

    fn set_overwrite(&self, state: bool) {
        self.overwrite.set(state);
    }
}

impl<LhsD: ?Sized, LhsG: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Backward
    for DivisionBackward<LhsD, LhsG, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    LhsG: Gradient,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsD::Dim>,
    LhsG::Dim: Dimension + DimMax<RhsG::Dim>,
{
    fn backward(&self) {
        let gradient = self.gradient();
        let mut buffer = expect_tensor_mut(&self.buffer);

        Zip::from(&mut *buffer)
            .and(&*gradient)
            .and_broadcast(&*self.right_data.data())
            .for_each(|d, g, r| *d = g / r);
        let reduced = reduce(self.left_grad.gradient().raw_dim(), &buffer);
        push_gradient(&self.left_grad, &reduced);

        Zip::from(&mut *buffer)
            .and(&*gradient)
            .and_broadcast(&*self.left_data.data())
            .and_broadcast(&*self.right_data.data())
            .for_each(|d, g, l, r| *d = -g * l / r.powi(2));
        let reduced = reduce(self.right_grad.gradient().raw_dim(), &buffer);
        push_gradient(&self.right_grad, &reduced);
    }

    fn no_grad(&self) {
        *self.gradient.borrow_mut() = None;
    }

    fn with_grad(&self) {
        *self.gradient.borrow_mut() = Some(Tensor::zeros(self.shape.clone()));
    }
}

impl<LhsD: ?Sized, LhsG: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Debug
    for DivisionBackward<LhsD, LhsG, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    LhsG: Gradient,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsD::Dim>,
    LhsG::Dim: Dimension + DimMax<RhsG::Dim>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("DivisionBackward")
            .field("gradient", &self.gradient.borrow())
            .field("overwrite", &self.overwrite.get())
            .finish()
    }
}

impl<LhsD: ?Sized, LhsG: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Display
    for DivisionBackward<LhsD, LhsG, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    LhsG: Gradient,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsD::Dim>,
    LhsG::Dim: Dimension + DimMax<RhsG::Dim>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &*self.gradient.borrow() {
            Some(gradient) => write!(f, "{}", gradient),
            None => write!(f, "None"),
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ DivisionBackwardLeft ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub struct DivisionBackwardLeft<LhsG: ?Sized, RhsD: ?Sized>
where
    RhsD: Data,
    LhsG: Gradient,
    LhsG::Dim: Dimension + DimMax<RhsD::Dim>,
{
    gradient: RefCell<Option<BroadTensor<LhsG::Dim, RhsD::Dim>>>,
    shape: Broadcasted<LhsG::Dim, RhsD::Dim>,
    overwrite: Cell<bool>,
    buffer: RefCell<Option<BroadTensor<LhsG::Dim, RhsD::Dim>>>,
    left_grad: Rc<LhsG>,
    right_data: Rc<RhsD>,
}

impl<LhsG: ?Sized, RhsD: ?Sized> DivisionBackwardLeft<LhsG, RhsD>
where
    RhsD: Data,
    LhsG: Gradient,
    LhsG::Dim: Dimension + DimMax<RhsD::Dim>,
{
    pub fn new(left_grad: Rc<LhsG>, right_data: Rc<RhsD>) -> Self {
        let gradient = cobroadcasted_zeros(&left_grad.gradient(), &right_data.data());
        let shape = gradient.raw_dim();

        Self {
            gradient: RefCell::new(Some(gradient)),
            shape: shape.clone(),
            overwrite: Cell::new(true),
            buffer: RefCell::new(Some(Tensor::zeros(shape))),
            left_grad,
            right_data,
        }
    }
}

impl<LhsG: ?Sized, RhsD: ?Sized> Gradient for DivisionBackwardLeft<LhsG, RhsD>
where
    RhsD: Data,
    LhsG: Gradient,
    LhsG::Dim: Dimension + DimMax<RhsD::Dim>,
{
    type Dim = Broadcasted<LhsG::Dim, RhsD::Dim>;

    fn gradient(&self) -> Ref<Tensor<Self::Dim>> {
        expect_tensor(&self.gradient)
    }

    fn gradient_mut(&self) -> RefMut<Tensor<Self::Dim>> {
        expect_tensor_mut(&self.gradient)
    }
}

impl<LhsG: ?Sized, RhsD: ?Sized> Overwrite for DivisionBackwardLeft<LhsG, RhsD>
where
    RhsD: Data,
    LhsG: Gradient,
    LhsG::Dim: Dimension + DimMax<RhsD::Dim>,
{
    fn can_overwrite(&self) -> bool {
        self.overwrite.get()
    }

    fn set_overwrite(&self, state: bool) {
        self.overwrite.set(state);
    }
}

impl<LhsG: ?Sized, RhsD: ?Sized> Backward for DivisionBackwardLeft<LhsG, RhsD>
where
    RhsD: Data,
    LhsG: Gradient,
    LhsG::Dim: Dimension + DimMax<RhsD::Dim>,
{
    fn backward(&self) {
        let gradient = self.gradient();
        let mut buffer = expect_tensor_mut(&self.buffer);

        Zip::from(&mut *buffer)
            .and(&*gradient)
            .and_broadcast(&*self.right_data.data())
            .for_each(|d, g, r| *d = g / r);
        let reduced = reduce(self.left_grad.gradient().raw_dim(), &buffer);
        push_gradient(&self.left_grad, &reduced);
    }

    fn no_grad(&self) {
        *self.gradient.borrow_mut() = None;
    }

    fn with_grad(&self) {
        *self.gradient.borrow_mut() = Some(Tensor::zeros(self.shape.clone()));
    }
}

impl<LhsG: ?Sized, RhsD: ?Sized> Debug for DivisionBackwardLeft<LhsG, RhsD>
where
    RhsD: Data,
    LhsG: Gradient,
    LhsG::Dim: Dimension + DimMax<RhsD::Dim>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("DivisionBackwardLeft")
            .field("gradient", &self.gradient.borrow())
            .field("overwrite", &self.overwrite.get())
            .finish()
    }
}

impl<LhsG: ?Sized, RhsD: ?Sized> Display for DivisionBackwardLeft<LhsG, RhsD>
where
    RhsD: Data,
    LhsG: Gradient,
    LhsG::Dim: Dimension + DimMax<RhsD::Dim>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &*self.gradient.borrow() {
            Some(gradient) => write!(f, "{}", gradient),
            None => write!(f, "None"),
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ DivisionBackwardRight ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub struct DivisionBackwardRight<LhsD: ?Sized, RhsD: ?Sized, RhsG: ?Sized>
where
    LhsD: Data,
    RhsD: Data,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsG::Dim>,
{
    gradient: RefCell<Option<BroadTensor<LhsD::Dim, RhsG::Dim>>>,
    shape: Broadcasted<LhsD::Dim, RhsG::Dim>,
    overwrite: Cell<bool>,
    buffer: RefCell<Option<BroadTensor<LhsD::Dim, RhsG::Dim>>>,
    left_data: Rc<LhsD>,
    right_data: Rc<RhsD>,
    right_grad: Rc<RhsG>,
}

impl<LhsD: ?Sized, RhsD: ?Sized, RhsG: ?Sized> DivisionBackwardRight<LhsD, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsG::Dim>,
{
    /// Creates a new `DivisionBackwardLeft` node whose operands are `left_data`, `right_data` and
    /// `right_grad`.
    pub fn new(left_data: Rc<LhsD>, right_data: Rc<RhsD>, right_grad: Rc<RhsG>) -> Self {
        let gradient = cobroadcasted_zeros(&left_data.data(), &right_grad.gradient());
        let shape = gradient.raw_dim();

        Self {
            gradient: RefCell::new(Some(gradient)),
            shape: shape.clone(),
            overwrite: Cell::new(true),
            buffer: RefCell::new(Some(Tensor::zeros(shape))),
            left_data,
            right_data,
            right_grad,
        }
    }
}

impl<LhsD: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Gradient for DivisionBackwardRight<LhsD, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsG::Dim>,
{
    type Dim = Broadcasted<LhsD::Dim, RhsG::Dim>;

    fn gradient(&self) -> Ref<Tensor<Self::Dim>> {
        expect_tensor(&self.gradient)
    }

    fn gradient_mut(&self) -> RefMut<Tensor<Self::Dim>> {
        expect_tensor_mut(&self.gradient)
    }
}

impl<LhsD: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Overwrite for DivisionBackwardRight<LhsD, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsG::Dim>,
{
    fn can_overwrite(&self) -> bool {
        self.overwrite.get()
    }

    fn set_overwrite(&self, state: bool) {
        self.overwrite.set(state);
    }
}

impl<LhsD: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Backward for DivisionBackwardRight<LhsD, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsG::Dim>,
{
    fn backward(&self) {
        let gradient = self.gradient();
        let mut buffer = expect_tensor_mut(&self.buffer);

        Zip::from(&mut *buffer)
            .and(&*gradient)
            .and_broadcast(&*self.left_data.data())
            .and_broadcast(&*self.right_data.data())
            .for_each(|d, g, l, r| *d = -g * l / r.powi(2));
        let reduced = reduce(self.right_grad.gradient().raw_dim(), &buffer);
        push_gradient(&self.right_grad, &reduced);
    }

    fn no_grad(&self) {
        *self.gradient.borrow_mut() = None;
    }

    fn with_grad(&self) {
        *self.gradient.borrow_mut() = Some(Tensor::zeros(self.shape.clone()));
    }
}

impl<LhsD: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Debug for DivisionBackwardRight<LhsD, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsG::Dim>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("DivisionBackwardRight")
            .field("gradient", &self.gradient.borrow())
            .field("overwrite", &self.overwrite.get())
            .finish()
    }
}

impl<LhsD: ?Sized, RhsD: ?Sized, RhsG: ?Sized> Display for DivisionBackwardRight<LhsD, RhsD, RhsG>
where
    LhsD: Data,
    RhsD: Data,
    RhsG: Gradient,
    LhsD::Dim: Dimension + DimMax<RhsG::Dim>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &*self.gradient.borrow() {
            Some(gradient) => write!(f, "{}", gradient),
            None => write!(f, "None"),
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Tests ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cfg(test)]
mod test;
