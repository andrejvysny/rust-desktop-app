//! A replaceable UI that changes application state only through commands.
mod app;
pub mod control;
mod menus;
mod panels;
mod theme;

pub use app::AppUi;

#[cfg(test)]
mod tests;
