//! Search: one pipeline over offers from the index and from live store searches.

mod engine;
mod text;

pub use self::{
    engine::{Filters, Results, Sort, assemble},
    text::{TextQuery, normalize},
};
