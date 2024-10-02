#![feature(test)]
#![feature(substr_range)]
#![feature(stmt_expr_attributes)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::perf)]

pub mod request;
pub mod response;
pub mod server;
pub mod threadpool;
