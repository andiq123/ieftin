//! Fetching store pages politely: robots.txt is obeyed and requests are spaced out.

mod fetcher;
mod robots;

pub use self::fetcher::{FetchError, Fetcher};
