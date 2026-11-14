//! The AI Workspace engine: workspace layout, configuration, projects,
//! validation, permission sessions, context building, search and the local
//! runtime. It returns data and never prints, reads the terminal, or names a
//! front end's command; front ends such as the `ai-workspace` CLI render
//! what it returns. See ADR-0008 in `DECISIONS.md`.

pub mod config;
pub mod context;
pub mod error;
pub mod paths;
pub mod runtime;
pub mod search;
pub mod workspace;

mod engine;

pub use engine::{Engine, InitReport, SetRootReport, set_workspace_root};
pub use error::{AppError, GatedOperation, Result};
