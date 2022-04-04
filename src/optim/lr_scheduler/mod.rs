//! Learning rate schedulers.
//!
//! Learning rate scheduling should be applied after optimizer’s update;
//! for instance, you should write your code this way:
//!
//! ```
//! # use neuronika::optim;
//! # use neuronika::optim::Optimizer;
//! # use neuronika::optim::lr_scheduler::LRScheduler;
//! # const EPOCHS: usize = 5;
//! # let optim = optim::SGD::new(vec![], 1., optim::L2::new(0.1));
//! # let scheduler = optim::lr_scheduler::LambdaLR::new(&optim, |epoch| epoch as f32);
//! # let mut loss = neuronika::ones(1).requires_grad() + 0.;
//! for epoch in 0..EPOCHS {
//!    loss.forward();
//!    loss.backward(1.0);
//!    optim.step();
//!    optim.zero_grad();
//!    scheduler.step();
//! }
//! ```
//!
//! Learning rate schedulers can be chained together. The result is that each scheduler is applied
//! one after the other on the learning rate obtained by the one preceding it.
//!
//! ```
//! # use neuronika::optim;
//! # use neuronika::optim::{SGD,Optimizer, L2};
//! # use neuronika::optim::lr_scheduler::{LRScheduler, LambdaLR, MultiplicativeLR};
//! # const EPOCHS: usize = 5;
//! let optim = SGD::new(vec![], 0.01, L2::new(0.1));
//! let scheduler1 = LambdaLR::new(&optim, |epoch| 1.0_f32 / epoch as f32);
//! let scheduler2 = MultiplicativeLR::new(&optim, |epoch| 0.1_f32 * epoch as f32);
//! # let mut loss = neuronika::ones(1).requires_grad() + 0.;
//!
//! for epoch in 0..EPOCHS {
//!    loss.forward();
//!    loss.backward(1.0);
//!    optim.step();
//!    optim.zero_grad();
//!    scheduler1.step();
//!    scheduler2.step();
//! }
//! ```

mod exponential_lr;
mod lambda_lr;
mod multi_step_lr;
mod multiplicative_lr;
mod step_lr;

use std::cell::Cell;

pub use exponential_lr::*;
pub use lambda_lr::*;
pub use multi_step_lr::*;
pub use multiplicative_lr::*;
pub use step_lr::*;

/// Learning rate scheduler trait, defines the scheduler's logic.
pub trait LRScheduler {
    /// Updates the learning rate.
    fn step(&self);

    /// Returns an immutable reference to the last computed learning rate.
    fn get_last_lr(&self) -> f32;

    /// Returns an immutable reference to the current learning rate.
    fn get_current_lr(&self) -> f32;

    /// Returns an immutable reference to the current epoch.
    fn get_current_epoch(&self) -> usize;

    /// Sets the current epoch.
    fn set_current_epoch(&self, epoch: usize);

    /// Prints the update of the learning rate. It should be called after `.step()`.
    fn print_lr(&self) {
        println!(
            "epoch {}: learning rate adjusted to [{}]",
            self.get_current_epoch(),
            self.get_current_lr()
        );
    }
}

/// Prepares a learning rate scheduler to perform the next update step.
///
/// Sets `last_lr` as `current_lr` and increases `current_epoch`.
fn prepare_step(last_lr: &Cell<f32>, current_lr: &Cell<f32>, current_epoch: &Cell<usize>) {
    // Set current learning rate as last learning rate.
    last_lr.set(current_lr.get());
    // Set current epoch as last epoch.
    let last_epoch = current_epoch.get();
    // Increase current epoch.
    current_epoch.set(last_epoch + 1);
}

#[cfg(test)]
mod test;
