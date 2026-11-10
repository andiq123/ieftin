//! Ranking: matching offers to the query and turning them into one page of results.

mod engine;
mod text;

pub use self::{
    engine::{Filters, Results, Sort, assemble},
    text::{TextQuery, normalize},
};
