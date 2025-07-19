use vtiger_client::Vtiger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vtiger = Vtiger::new("https://demo.vtiger.com", "admin", "your_access_key");

    // Get current user info
    let user_info = vtiger.me().await?;
    if user_info.success {
        println!("Authentication successful");
        println!("User info: {:#?}", user_info.result);
    } else {
        println!("Authentication failed: {:?}", user_info.error);
    }

    // List available modules
    let modules = vtiger.list_types(&[]).await?;
    if modules.success {
        println!("Available modules: {:#?}", modules.result);
    }

    // Query some data
    let leads = vtiger.query("SELECT * FROM Leads LIMIT 5").await?;
    if leads.success {
        println!("Found {} leads", leads.result.unwrap_or_default().len());
    }

    Ok(())
}
