#![warn(clippy::all)]
#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
//! # Vtiger Client
//!
//! A Rust client library for the Vtiger CRM REST API.
//!
//! This crate provides a simple and ergonomic interface for interacting with
//! Vtiger CRM instances through their REST API.
//!
//! ## Features
//!
//! - Full async/await support
//! - Type-safe API responses
//! - Batch export functionality
//! - Support for JSON, JSON Lines, and CSV export formats
//! - Comprehensive error handling
//!
//! ## Quick Start
//!
//! ```no_run
//! use vtiger_client::Vtiger;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let vtiger = Vtiger::new(
//!         "https://your-instance.vtiger.com",
//!         "your_username",
//!         "your_access_key"
//!     );
//!
//!     // Get user info
//!     let user_info = vtiger.me().await?;
//!     println!("User info: {:?}", user_info);
//!
//!     // Query records
//!     let leads = vtiger.query("SELECT * FROM Leads LIMIT 10").await?;
//!     println!("Found {} leads", leads.result.unwrap_or_default().len());
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod types;

// Re-export the main types for convenience
pub use client::Vtiger;
pub use types::{ApiError, ExportFormat, VtigerQueryResponse, VtigerResponse};
