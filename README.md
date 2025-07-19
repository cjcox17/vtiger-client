# Vtiger Client for Rust

A modern, async Rust client library for the Vtiger CRM REST API.

> [!IMPORTANT]
> This is a work in progress, it only has basic functionality for:
> - User information retrieval (/me)
> - Module description (/describe)
> - Record querying (/query)
> - Record creation (/create)
> - Nice exporting!

## Features

- **Async/await support** - Built on tokio and reqwest
- **Batch operations** - Efficient bulk data export to types, JSON, JSON Lines, and CSV.
- **Well documented** - Extensive API documentation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vtiger-client = "0.1"
tokio = { version = "1.46.1", features = ["full"] }
reqwest = { version = "0.12.22", features = ["json", "rustls-tls"] }
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.140"
csv = "1.3.1"
futures = "0.3.31"
```

## Quick Start
```rust
use vtiger_client::Vtiger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vtiger = Vtiger::new(
        "https://your-instance.vtiger.com",
        "your_username",
        "your_access_key"
    );

    // Get user info
    let user_info = vtiger.me().await?;
    println!("User: {:?}", user_info);

    // Query records
    let leads = vtiger.query("SELECT * FROM Leads LIMIT 10").await?;
    println!("Leads: {:?}", leads);

    Ok(())
}
```

## Authentication
Get your access key from Vtiger:

- Log into your Vtiger instance
- Go to My Preferences → Security
- Copy your Access Key

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
