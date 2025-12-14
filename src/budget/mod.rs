//! Budget module - cost tracking and model pricing.
//!
//! # Key Concepts
//! - Budget: tracks total and allocated costs for a task
//! - Pricing: fetches and caches OpenRouter model pricing
//! - Allocation: algorithms for distributing budget across subtasks

mod budget;
mod pricing;
mod allocation;

pub use budget::{Budget, BudgetError};
pub use pricing::{ModelPricing, PricingInfo};
pub use allocation::{AllocationStrategy, allocate_budget};

