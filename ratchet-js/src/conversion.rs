use crate::JsExecutionError;
use boa_engine::{property::PropertyKey, Context as BoaContext, JsString, Source};
use serde_json::Value as JsonValue;
use tracing::{debug, trace};

/// Prepare input data for JavaScript execution
pub fn prepare_input_argument(
    context: &mut BoaContext,
    input_data: &JsonValue,
) -> Result<boa_engine::JsValue, JsExecutionError> {
    trace!("Converting input data to JavaScript format");
    let input_js_str =
        serde_json::to_string(input_data).map_err(|e| JsExecutionError::InvalidOutputFormat(e.to_string()))?;

    trace!("Parsing input JSON string into JavaScript object");
    context
        .eval(Source::from_bytes(&format!(
            "JSON.parse('{}')",
            input_js_str.replace("'", "\\'")
        )))
        .map_err(|e| JsExecutionError::ExecutionError(format!("Failed to parse input JSON: {}", e)))
}

/// Convert JavaScript result to JSON
pub fn convert_js_result_to_json(
    context: &mut BoaContext,
    result: boa_engine::JsValue,
) -> Result<JsonValue, JsExecutionError> {
    debug!("Converting JavaScript result back to JSON");

    // Set temporary variable to hold the result so we can stringify it
    context
        .global_object()
        .set(
            PropertyKey::from(JsString::from("__temp_result")),
            result,
            true,
            context,
        )
        .map_err(|e| JsExecutionError::ExecutionError(format!("Failed to set temporary result: {}", e)))?;

    // Convert to JSON string
    let result_json_str = context
        .eval(Source::from_bytes("JSON.stringify(__temp_result)"))
        .map_err(|e| JsExecutionError::ExecutionError(format!("Failed to stringify result: {}", e)))?;

    // Convert to Rust string
    let result_str = result_json_str
        .to_string(context)
        .map_err(|e| JsExecutionError::InvalidOutputFormat(e.to_string()))?;

    let json_str = result_str.to_std_string_escaped();

    // Parse the JSON string into a JsonValue
    serde_json::from_str(&json_str).map_err(|e| JsExecutionError::InvalidOutputFormat(e.to_string()))
}

/// Set a JavaScript value in the global context
pub fn set_js_value(context: &mut BoaContext, variable_name: &str, value: &JsonValue) -> Result<(), JsExecutionError> {
    let value_str = serde_json::to_string(value).map_err(|e| JsExecutionError::InvalidOutputFormat(e.to_string()))?;

    let js_code = format!("var {} = {};", variable_name, value_str);
    context
        .eval(Source::from_bytes(&js_code))
        .map_err(|e| JsExecutionError::ExecutionError(format!("Failed to set variable {}: {}", variable_name, e)))?;

    Ok(())
}
