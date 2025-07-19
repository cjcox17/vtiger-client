use vtiger_client::{ExportFormat, Vtiger};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vtiger = Vtiger::new(
        "https://your-instance.vtiger.com",
        "your_username",
        "your_access_key",
    );

    // Export leads with filtering
    let prefixes: Vec<String> = vec![
        "Chicago".to_string(),
        "Seattle".to_string(),
        "California".to_string(),
    ];
    vtiger
        .export(
            "Leads",
            Some(("location", prefixes)),
            200,
            3,
            vec![ExportFormat::Json, ExportFormat::CSV],
        )
        .await?;

    println!("Export completed! Check output.json and output.csv");
    Ok(())
}
