use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Supported export formats for Vtiger data.
///
/// Each format has different characteristics:
/// - JSON creates a single file with all records in an array
/// - JSON Lines creates one JSON object per line (streaming-friendly)
/// - CSV creates a traditional spreadsheet format with headers
///
/// # Examples
///
/// ```no_run
/// use vtiger_client::ExportFormat;
///
/// let formats = vec![
///     ExportFormat::Json,
///     ExportFormat::JsonLines,
///     ExportFormat::CSV,
/// ];
/// ```
#[derive(Debug, PartialEq)]
pub enum ExportFormat {
    /// JSON format (.json) - exports data as a single JSON array
    Json,
    /// JSON Lines format (.jsonl) - exports each record as a separate JSON object on its own line
    JsonLines,
    /// CSV format (.csv) - exports data as comma-separated values with headers
    CSV,
}

/// Response wrapper for all Vtiger REST API calls.
///
/// This struct represents the standard response format returned by the Vtiger API.
/// All API responses follow this structure, containing a success flag, optional result data,
/// and optional error information.
///
/// # Fields
///
/// * `success` - Indicates whether the API call was successful
/// * `result` - Contains the response data when the call succeeds
/// * `error` - Contains error details when the call fails
///
/// # Examples
///
/// ```no_run
/// use vtiger_client::{ApiError, VtigerResponse};
/// use indexmap::IndexMap;
///
/// // Successful response
/// let success_response = VtigerResponse {
///     success: true,
///     result: Some(IndexMap::new()),
///     error: None,
/// };
///
/// // Error response
/// let error_response = VtigerResponse {
///     success: false,
///     result: None,
///     error: Some(ApiError {
///         code: "404".to_string(),
///         message: "Not Found".to_string(),
///     }),
/// };
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct VtigerResponse {
    /// Indicates whether the API call was successful
    pub success: bool,
    /// Contains the response data when `success` is true
    pub result: Option<IndexMap<String, serde_json::Value>>,
    /// Contains error details when `success` is false
    pub error: Option<ApiError>,
}

/// Response format for Vtiger query operations that return multiple records.
///
/// This struct is used specifically for API calls that return arrays of data,
/// such as `SELECT` queries or list operations. It follows the same success/error
/// pattern as [`VtigerResponse`] but contains a vector of records instead of a single object.
///
/// # Examples
///
/// ```no_run
/// use indexmap::IndexMap;
/// use serde_json::json;
/// use vtiger_client::VtigerQueryResponse;
///
/// // Query response with multiple records
/// let response = VtigerQueryResponse {
///     success: true,
///     result: Some(vec![
///         IndexMap::from([("id".to_string(), json!("1"))]),
///         IndexMap::from([("id".to_string(), json!("2"))]),
///     ]),
///     error: None,
/// };
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct VtigerQueryResponse {
    /// Whether the query operation succeeded
    pub success: bool,
    /// Array of records returned by the query (present when successful)
    pub result: Option<Vec<IndexMap<String, serde_json::Value>>>,
    /// Error details (present when failed)
    pub error: Option<ApiError>,
}

/// Error information returned by the Vtiger API.
///
/// When an API call fails, the Vtiger API returns structured error information
/// containing both a code for programmatic handling and a human-readable message.
///
/// # Examples
///
/// ```no_run
/// use vtiger_client::ApiError;
/// let error = ApiError {
///     code: "AUTHENTICATION_FAILURE".to_string(),
///     message: "Invalid username or access key".to_string(),
/// };
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    /// Error code for programmatic error handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
}
