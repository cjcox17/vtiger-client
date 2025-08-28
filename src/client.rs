use csv::{Writer, WriterBuilder};
use futures::stream::{self, StreamExt};
use indexmap::IndexMap;
use reqwest::{self, Response};
use std::collections::HashMap;
use std::{fs::File, io::Write};
use tokio::sync::mpsc;

use crate::types::{ExportFormat, VtigerQueryResponse, VtigerResponse};
/// The base endpoint path for the Vtiger REST API.
///
/// This path is appended to the Vtiger instance URL to form the complete
/// API endpoint. All REST API operations use this base path.
///
/// # Example
///
/// With a Vtiger instance at `https://example.vtiger.com/`, the full
/// API URL would be: `https://example.vtiger.com/restapi/v1/vtiger/default`
const LINK_ENDPOINT: &str = "restapi/v1/vtiger/default";

/// Client for interacting with the Vtiger REST API.
///
/// This struct provides a high-level interface for making authenticated requests
/// to a Vtiger CRM instance. It handles authentication, request formatting, and
/// response parsing automatically.
///
/// # Examples
///
/// ```no_run
///
/// # #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     use vtiger_client::Vtiger;
///
///     let vtiger = Vtiger::new(
///         "https://your-instance.vtiger.com",
///         "your_username",
///         "your_access_key"
///     );
///
///     // Query records
///     let response = vtiger.query("SELECT * FROM Leads LIMIT 10").await?;
///     Ok(())
/// }
/// ```
///
/// # Authentication
///
/// Authentication uses the Vtiger username and access key. The access key can be
/// found in your Vtiger user preferences under "My Preferences" > "Security".
#[derive(Debug)]
pub struct Vtiger {
    /// The base URL of the Vtiger instance
    url: String,
    /// Username for API authentication
    username: String,
    /// Access key for API authentication (found in user preferences)
    access_key: String,
    /// HTTP client for making requests
    client: reqwest::Client,
}

impl Vtiger {
    /// Creates a new Vtiger API client.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL of your Vtiger instance (e.g., `https://your-instance.vtiger.com`)
    /// * `username` - Your Vtiger username
    /// * `access_key` - Your Vtiger access key (found in My Preferences > Security)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vtiger_client::Vtiger;
    ///
    /// let vtiger = Vtiger::new(
    ///     "https://demo.vtiger.com",
    ///     "admin",
    ///     "your_access_key_here"
    /// );
    /// ```
    pub fn new(url: &str, username: &str, access_key: &str) -> Self {
        Vtiger {
            url: url.to_string(),
            username: username.to_string(),
            access_key: access_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Performs an authenticated GET request to the Vtiger API.
    ///
    /// This is an internal method that handles authentication and common headers
    /// for all GET operations. The URL is constructed by combining the base URL,
    /// the API endpoint path, and the provided endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The specific API endpoint path (e.g., "/query")
    /// * `query` - Query parameters as key-value pairs
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the HTTP response or a request error.
    async fn get(
        &self,
        endpoint: &str,
        query: &[(&str, &str)],
    ) -> Result<Response, reqwest::Error> {
        let url = format!("{}{}{}", self.url, LINK_ENDPOINT, endpoint);
        let mut request = self.client.get(&url);
        if !query.is_empty() {
            request = request.query(query);
        }
        request
            .basic_auth(&self.username, Some(&self.access_key))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .send()
            .await
    }

    /// Performs an authenticated POST request to the Vtiger API.
    ///
    /// This is an internal method that handles authentication and common headers
    /// for all POST operations. The URL is constructed by combining the base URL,
    /// the API endpoint path, and the provided endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The specific API endpoint path (e.g., "/create")
    /// * `query` - Query parameters as key-value pairs
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the HTTP response or a request error.
    async fn post(
        &self,
        endpoint: &str,
        query: &[(&str, &str)],
    ) -> Result<Response, reqwest::Error> {
        let url = format!("{}{}{}", self.url, LINK_ENDPOINT, endpoint);
        let mut request = self.client.post(&url);
        if !query.is_empty() {
            request = request.query(query);
        }
        request
            .basic_auth(&self.username, Some(&self.access_key))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .send()
            .await
    }

    /// Retrieves information about the currently authenticated user.
    ///
    /// This method calls the `/me` endpoint to get details about the user account
    /// associated with the provided credentials. It's useful for verifying authentication
    /// and retrieving user-specific information.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a [`VtigerResponse`] with user information on success,
    /// or a `reqwest::Error` if the request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vtiger_client::Vtiger;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let vtiger = Vtiger::new(
    ///         "https://demo.vtiger.com",
    ///         "admin",
    ///         "your_access_key",
    ///     );
    ///
    ///     let user_info = vtiger.me().await?;
    ///     if user_info.success {
    ///         println!("Authenticated as: {:?}", user_info.result);
    ///     } else {
    ///         eprintln!("Authentication failed: {:?}", user_info.error);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The network request fails
    /// - The response cannot be parsed as JSON
    /// - The server returns an invalid response format
    pub async fn me(&self) -> Result<VtigerResponse, reqwest::Error> {
        let response = self.get("/me", &[]).await?;
        let vtiger_response = response.json::<VtigerResponse>().await?;
        Ok(vtiger_response)
    }

    /// List the available modules and their information
    ///
    /// This endpoint returns a list of all of the available modules (like Contacts, Leads, Accounts)
    /// available in your vtiger instance. You can optionally filter modules by the types of fields
    /// they contain.
    ///
    /// #Arguments
    ///
    /// * `query`: An optional query string to filter the modules by. Common parameters:
    /// - `("fieldTypeList", "null") or &[]- Return all available modules
    /// - `("fieldTypeList", "picklist")- Return modules with picklist fields
    /// - `("fieldTypeList", "grid")- Return modules with grid fields
    ///
    /// # Returns
    ///
    /// A `VtigerResponse` containing module information. The `result` typically includes:
    /// - `types`: Array of module names
    /// - `information`: Object with detailed info about each module
    ///
    /// # Errors
    ///
    /// Returns `reqwest::Error` for network issues or authentication failures.
    /// Check the response's `success` field and `error` field for API-level errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use vtiger_client::Vtiger;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let vtiger = Vtiger::new("https://demo.vtiger.com/", "admin", "access_key");
    ///
    /// // Get all available modules
    /// let all_modules = vtiger.list_types(&[("fieldTypeList", "null")]).await?;
    ///
    /// // Get modules with picklist fields only
    /// let picklist_modules = vtiger.list_types(&[("fieldTypeList[]", "picklist")]).await?;
    ///
    /// // Get all modules (no filtering)
    /// let modules = vtiger.list_types(&[]).await?;
    ///
    /// if all_modules.success {
    ///     println!("Available modules: {:#?}", all_modules.result);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_types(
        &self,
        query: &[(&str, &str)],
    ) -> Result<VtigerResponse, reqwest::Error> {
        let response = self.get("/listtypes", query).await?;
        let vtiger_response = response.json::<VtigerResponse>().await?;
        Ok(vtiger_response)
    }
    /// List the fields of a module.
    ///
    /// This endpoint returns a list of fields, their types, labels, default values, and other metadata of a module.
    ///
    /// #Arguments
    ///
    /// - `module_name`: The name of the module to describe.
    ///
    /// #Errors
    ///
    /// Returns `reqwest::Error` for network issues or authentication failures.
    /// Check the response's `success` field and `error` field for API-level errors.
    ///
    /// #Examples
    ///
    /// ```no_run
    /// # use vtiger_client::Vtiger;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let vtiger = Vtiger::new("https://demo.vtiger.com/", "admin", "access_key");
    ///
    /// // Describe the Accounts module
    /// let accounts_description = vtiger.describe("Accounts").await?;
    ///
    /// if accounts_description.success {
    ///     println!("Fields of Accounts module: {:#?}", accounts_description.result);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn describe(&self, module_name: &str) -> Result<VtigerResponse, reqwest::Error> {
        let response = self
            .get("/describe", &[("elementType", module_name)])
            .await?;
        let vtiger_response = response.json::<VtigerResponse>().await?;
        Ok(vtiger_response)
    }

    /// Export records from a module to multiple file formats.
    ///
    /// This method performs a high-performance, concurrent export of records from a Vtiger module.
    /// It automatically handles batching, concurrency, and multiple output formats simultaneously.
    /// Be wary of rate limits and potential API throttling.
    ///
    /// # Process Overview
    ///
    /// 1. **Count Phase**: Determines record counts for each filter
    /// 2. **Batch Phase**: Creates work items based on batch size
    /// 3. **Concurrent Export**: Runs multiple queries in parallel
    /// 4. **File Writing**: Writes to multiple formats simultaneously
    ///
    /// # Arguments
    ///
    /// * `module` - The Vtiger module name (e.g., "Leads", "Contacts", "Accounts")
    /// * `query_filter` - Optional filter as (column_name, filter_values). Uses LIKE queries with '-' suffix
    /// * `batch_size` - Number of records per batch (recommended: 100-200)
    /// * `concurrency` - Number of concurrent requests (recommended: 2-5)
    /// * `format` - Vector of output formats to generate simultaneously
    ///
    /// # Output Files
    ///
    /// * **JSON**: `output.json` - Single JSON array with all records
    /// * **JSON Lines**: `output.jsonl` - One JSON object per line
    /// * **CSV**: `output.csv` - Traditional CSV with headers
    ///
    /// # Performance Notes
    ///
    /// * Vtiger's query API has an implicit limit of ~100 records per query
    /// * Batch sizes of 200 may work depending on module and account tier
    /// * Higher concurrency may hit API rate limits
    /// * Multiple formats are written simultaneously for efficiency
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vtiger_client::{Vtiger, ExportFormat};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let vtiger = Vtiger::new("https://demo.vtiger.com", "admin", "key");
    ///
    ///     // Export all leads
    ///     vtiger.export(
    ///         "Leads",
    ///         None,
    ///         200,
    ///         3,
    ///         vec![ExportFormat::Json, ExportFormat::CSV]
    ///     ).await?;
    ///
    ///     // Export leads with specific locations
    ///     let locations = vec!["Los Angeles".to_string(), "Seattle".to_string()];
    ///     vtiger.export(
    ///         "Leads",
    ///         Some(("Location", locations)),
    ///         200,
    ///         3,
    ///         vec![ExportFormat::JsonLines]
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Network requests fail
    /// * File creation fails
    /// * JSON serialization fails
    /// * Invalid module name or query syntax
    pub async fn export(
        &self,
        module: &str,
        query_filter: Option<(&str, Vec<String>)>,
        batch_size: usize,
        concurrency: usize,
        format: Vec<ExportFormat>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (column, query_filters) = match query_filter {
            Some((column, filters)) => (column, filters),
            None => ("", vec![]),
        };

        let query_filter_counts: Vec<(String, i32)> = stream::iter(query_filters)
            .map(|query_filter| async move {
                let query = format!(
                    "SELECT count(*) from {} WHERE {} LIKE '{}-%';",
                    module, column, query_filter
                );

                let count = match self.query(&query).await {
                    Ok(result) => match result.result {
                        Some(records) => {
                            // Extract the count value directly without chaining references
                            if let Some(first_record) = records.first() {
                                if let Some(count_val) = first_record.get("count") {
                                    if let Some(count_str) = count_val.as_str() {
                                        count_str.parse::<i32>().unwrap_or(0)
                                    } else {
                                        0
                                    }
                                } else {
                                    0
                                }
                            } else {
                                0
                            }
                        }
                        None => 0,
                    },
                    Err(_) => 0,
                };
                if count != 0 {
                    println!("Received {} records for {}", count, query_filter);
                }
                (query_filter, count)
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        let work_items = query_filter_counts
            .into_iter()
            .flat_map(|(query_filter, count)| {
                (0..count)
                    .step_by(batch_size)
                    .map(|offset| (query_filter.clone(), offset, batch_size))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let (record_sender, mut record_receiver) =
            mpsc::unbounded_channel::<IndexMap<String, serde_json::Value>>();

        let writer_task = tokio::spawn(async move {
            let mut file_json_lines: Option<File> = None;
            let mut file_json: Option<File> = None;
            let mut file_csv: Option<Writer<File>> = None;
            if format.contains(&ExportFormat::JsonLines) {
                file_json_lines = Some(File::create("output.jsonl")?);
            }
            if format.contains(&ExportFormat::Json) {
                file_json = Some(File::create("output.json")?);
                if let Some(file_json) = file_json.as_mut()
                    && let Err(e) = file_json.write_all(b"[")
                {
                    eprintln!("Could not write JSON array start: {e}");
                }
            }
            if format.contains(&ExportFormat::CSV) {
                file_csv = Some(
                    WriterBuilder::new()
                        .quote(b'"')
                        .quote_style(csv::QuoteStyle::Always)
                        .escape(b'\\')
                        .terminator(csv::Terminator::CRLF)
                        .from_path("output.csv")?,
                );
            }
            let mut count = 0;

            while let Some(record) = record_receiver.recv().await {
                if let Some(ref mut file) = file_json_lines {
                    writeln!(file, "{}", serde_json::to_string(&record)?)?;
                }
                if let Some(ref mut file) = file_json {
                    if count != 0 {
                        writeln!(file, ",")?;
                    }
                    writeln!(file, "{}", serde_json::to_string_pretty(&record)?)?;
                }
                if let Some(ref mut file) = file_csv {
                    if count == 0 {
                        let header: Vec<&str> = record.keys().map(|k| k.as_str()).collect();
                        file.write_record(header)?;
                    }
                    let values: Vec<String> = record
                        .values()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s
                                .replace('\n', "\\n")
                                .replace('\r', "\\r")
                                .replace('\t', "\\t"),
                            serde_json::Value::Number(s) => s.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                                serde_json::to_string(v).unwrap_or_else(|_| String::new())
                            }
                            _ => "".to_string(),
                        })
                        .collect();
                    file.write_record(values)?;
                }
                count += 1;
                if count % 10000 == 0 {
                    println!("Processed {} records", count);
                }
            }
            println!("Finished writing {} total records", count);

            if let Some(mut file) = file_json_lines {
                file.flush()?;
            }

            if let Some(mut file) = file_json {
                write!(file, "\n]")?; // Close the JSON array
                file.flush()?;
            }

            if let Some(mut file) = file_csv {
                file.flush()?;
            }

            println!("All files flushed and closed");
            Ok::<_, std::io::Error>(())
        });

        stream::iter(work_items)
            .map(|(query_filter, offset, batch_size)| {
                let sender = record_sender.clone();
                async move {
                    let query = format!(
                        "SELECT * FROM {} WHERE {} LIKE '{}%' LIMIT {}, {};",
                        module, column, query_filter, offset, batch_size
                    );
                    println!("Executing query: {}", query);

                    if let Ok(result) = self.query(&query).await
                        && let Some(records) = result.result
                    {
                        let record_count = records.len();
                        for record in records {
                            if sender.send(record).is_err() {
                                eprintln!("Failed to send record, writer may have stopped");
                                break;
                            }
                        }
                        println!(
                            "Sent {} records from {} offset {}",
                            record_count, query_filter, offset
                        );
                    }
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<_>>()
            .await;

        drop(record_sender);

        let _ = writer_task.await?;

        Ok(())
    }

    /// Create a new record in the specified module.
    ///
    /// This method creates a single record in Vtiger by sending field data
    /// to the `/create` endpoint. The fields are automatically serialized
    /// to JSON format as required by the Vtiger API.
    ///
    /// # Arguments
    ///
    /// * `module_name` - The name of the module to create the record in
    /// * `fields` - Array of field name/value pairs to set on the new record
    ///
    /// # Returns
    ///
    /// Returns a [`VtigerResponse`] containing the created record's ID and data
    /// on success, or the error details on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vtiger_client::Vtiger;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let vtiger = Vtiger::new("https://demo.vtiger.com", "admin", "key");
    ///
    ///     // Create a new lead
    ///     let response = vtiger.create(
    ///         "Leads",
    ///         &[
    ///             ("lastname", "Smith"),
    ///             ("firstname", "John"),
    ///             ("email", "john.smith@example.com"),
    ///             ("company", "Acme Corp"),
    ///         ]
    ///     ).await?;
    ///
    ///     if response.success {
    ///         println!("Created record: {:?}", response.result);
    ///     } else {
    ///         eprintln!("Creation failed: {:?}", response.error);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The network request fails
    /// * The response cannot be parsed as JSON
    /// * Invalid module name or field names are provided
    /// * Required fields are missing (check API response for details)
    pub async fn create(
        &self,
        module_name: &str,
        fields: &[(&str, &str)],
    ) -> Result<VtigerResponse, Box<dyn std::error::Error>> {
        let fields_map: HashMap<&str, &str> = fields.iter().cloned().collect();
        let element_json = serde_json::to_string(&fields_map)
            .map_err(|e| format!("Failed to serialize string IndexMap to JSON: {e}"))?;

        let response = self
            .post(
                "/create",
                &[("elementType", module_name), ("element", &element_json)],
            )
            .await?;
        let vtiger_response = response.json::<VtigerResponse>().await?;
        Ok(vtiger_response)
    }

    /// Retrieve a single record by its ID.
    ///
    /// This method fetches a complete record from Vtiger using its unique ID.
    /// The ID should be in Vtiger's format (e.g., "12x34" where 12 is the
    /// module ID and 34 is the record ID).
    ///
    /// # Arguments
    ///
    /// * `record_id` - The Vtiger record ID (format: "ModuleIDxRecordID")
    ///
    /// # Returns
    ///
    /// Returns a [`VtigerResponse`] containing the record data on success,
    /// or error details on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vtiger_client::Vtiger;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let vtiger = Vtiger::new("https://demo.vtiger.com", "admin", "key");
    ///
    ///     // Retrieve a specific lead record
    ///     let response = vtiger.retrieve("12x34").await?;
    ///
    ///     if response.success {
    ///         if let Some(record) = response.result {
    ///             println!("Record data: {:#?}", record);
    ///             // Access specific fields
    ///             if let Some(name) = record.get("lastname") {
    ///                 println!("Last name: {}", name);
    ///             }
    ///         }
    ///     } else {
    ///         eprintln!("Retrieval failed: {:?}", response.error);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Record ID Format
    ///
    /// Vtiger record IDs follow the format "ModuleIDxRecordID":
    /// * `12x34` - Module 12, Record 34
    /// * `4x567` - Module 4, Record 567
    ///
    /// You can typically get these IDs from:
    /// * Previous create/query operations
    /// * The Vtiger web interface URL
    /// * Other API responses
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The network request fails
    /// * The response cannot be parsed as JSON
    /// * The record ID format is invalid
    /// * The record doesn't exist or you don't have permission to view it
    pub async fn retrieve(&self, record_id: &str) -> Result<VtigerResponse, reqwest::Error> {
        let response = self.get("/retrieve", &[("id", record_id)]).await?;
        let vtiger_response = response.json::<VtigerResponse>().await?;
        Ok(vtiger_response)
    }

    /// Execute a SQL-like query against Vtiger data.
    ///
    /// This method allows you to query Vtiger records using a SQL-like syntax.
    /// It returns multiple records and is the primary method for searching
    /// and filtering data in Vtiger.
    ///
    /// # Arguments
    ///
    /// * `query` - SQL-like query string using Vtiger's query syntax
    ///
    /// # Returns
    ///
    /// Returns a [`VtigerQueryResponse`] containing an array of matching records
    /// on success, or error details on failure.
    ///
    /// # Query Syntax
    ///
    /// Vtiger supports a subset of SQL:
    /// * `SELECT * FROM ModuleName`
    /// * `SELECT field1, field2 FROM ModuleName`
    /// * `WHERE` conditions with `=`, `!=`, `LIKE`, `IN`
    /// * `ORDER BY` for sorting
    /// * `LIMIT` for pagination (recommended: ≤200 records)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vtiger_client::Vtiger;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let vtiger = Vtiger::new("https://demo.vtiger.com", "admin", "key");
    ///
    ///     // Basic query
    ///     let response = vtiger.query("SELECT * FROM Leads LIMIT 10").await?;
    ///
    ///     // Query with conditions
    ///     let response = vtiger.query(
    ///         "SELECT firstname, lastname, email FROM Leads WHERE email != ''"
    ///     ).await?;
    ///
    ///     // Query with LIKE operator
    ///     let response = vtiger.query(
    ///         "SELECT * FROM Leads WHERE lastname LIKE 'Smith%'"
    ///     ).await?;
    ///
    ///     // Query with ordering
    ///     let response = vtiger.query(
    ///         "SELECT * FROM Leads ORDER BY createdtime DESC LIMIT 50"
    ///     ).await?;
    ///
    ///     if response.success {
    ///         if let Some(records) = response.result {
    ///             println!("Found {} records", records.len());
    ///             for record in records {
    ///                 println!("Record: {:#?}", record);
    ///             }
    ///         }
    ///     } else {
    ///         eprintln!("Query failed: {:?}", response.error);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// * **Limit results**: Always use `LIMIT` to avoid timeouts
    /// * **Batch large queries**: Use pagination for large datasets
    /// * **Index usage**: Filter on indexed fields when possible
    /// * **Concurrent queries**: Use multiple queries for better performance
    ///
    /// # Common Query Patterns
    ///
    /// ```sql
    /// -- Get recent records
    /// SELECT * FROM Leads ORDER BY createdtime DESC LIMIT 100
    ///
    /// -- Filter by custom field
    /// SELECT * FROM Leads WHERE location LIKE 'Los Angeles%'
    ///
    /// -- Count records
    /// SELECT count(*) FROM Leads WHERE leadstatus = 'Hot'
    ///
    /// -- Multiple conditions
    /// SELECT * FROM Leads WHERE email != '' AND leadstatus IN ('Hot', 'Warm')
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The network request fails
    /// * The response cannot be parsed as JSON
    /// * Invalid SQL syntax is used
    /// * Referenced fields or modules don't exist
    /// * Query exceeds time or result limits
    pub async fn query(&self, query: &str) -> Result<VtigerQueryResponse, reqwest::Error> {
        let response = self.get("/query", &[("query", query)]).await?;
        let vtiger_response = response.json::<VtigerQueryResponse>().await?;
        Ok(vtiger_response)
    }

    /// Update a record in the database
    ///
    /// This function takes a single record that must include an ID and all
    /// required fields on the module in order to successfulyy update the record.
    /// It's recommended to use the revise function instead of this one.
    ///
    /// # Arguments
    ///
    /// * `fields` - A vector of tuples containing the field name and value to update.
    ///
    /// # Returns
    ///
    /// Returns a [`VtigerResponse`] containing a message indicating success or failure,
    /// and the result if successful.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vtiger_client::Vtiger;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let vtiger = Vtiger::new("https://demo.vtiger.com", "admin", "key");
    ///
    ///     // Query with conditions
    ///     let update_fields = vec![
    ///         ("id", "2x12345"),
    ///         ("first_name", "John"),
    ///         ("last_name", "Smith"),
    ///         ("email", "smith@example.com"),
    ///         ("phone", "604-555-1212"),
    ///         ("lane", "123 Main St"),
    ///         ("city", "Los Angeles"),
    ///         ("state", "CA"),
    ///         ("postal_code", "12345"),
    ///     ];
    ///     let response = vtiger.update(&update_fields).await?;
    ///
    ///     println!("Updated record: {:?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub async fn update(
        &self,
        fields: &Vec<(&str, &str)>,
    ) -> Result<VtigerResponse, Box<dyn std::error::Error>> {
        let map: HashMap<_, _> = fields.iter().cloned().collect();
        let fields_json = serde_json::to_string(&map)?;

        let response = self.post("/update", &[("element", &fields_json)]).await?;
        let vtiger_response = response.json::<VtigerResponse>().await?;
        Ok(vtiger_response)
    }

    /// Update a record in the database
    ///
    /// This function takes a single record that must include an ID and at least
    /// one field to update.
    ///
    /// # Arguments
    ///
    /// * `fields` - A vector of tuples containing the field name and value to update.
    ///
    /// # Returns
    ///
    /// Returns a [`VtigerResponse`] containing a message indicating success or failure,
    /// and the result if successful.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vtiger_client::Vtiger;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let vtiger = Vtiger::new("https://demo.vtiger.com", "admin", "key");
    ///
    ///     // Query with conditions
    ///     let update_fields = vec![
    ///         ("id", "2x12345"),
    ///         ("phone", "604-555-1212"),
    ///     ];
    ///     let response = vtiger.revise(&update_fields).await?;
    ///
    ///     println!("Updated record: {:?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub async fn revise(
        &self,
        fields: &Vec<(&str, &str)>,
    ) -> Result<VtigerResponse, Box<dyn std::error::Error>> {
        let map: HashMap<_, _> = fields.iter().cloned().collect();
        let fields_json = serde_json::to_string(&map)?;

        let response = self.post("/revise", &[("element", &fields_json)]).await?;
        let vtiger_response = response.json::<VtigerResponse>().await?;
        Ok(vtiger_response)
    }
}
