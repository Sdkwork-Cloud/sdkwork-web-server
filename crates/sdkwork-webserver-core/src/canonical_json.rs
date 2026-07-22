use sdkwork_utils_rust::crypto::sha256_hash;
use serde::Serialize;
use serde_json::Value;

pub(crate) fn canonical_sha256_excluding_field(
    value: &impl Serialize,
    excluded_field: &str,
) -> Result<String, serde_json::Error> {
    let mut value = serde_json::to_value(value)?;
    value
        .as_object_mut()
        .expect("typed runtime snapshots always serialize as objects")
        .remove(excluded_field);
    let mut canonical = String::new();
    write_canonical_json(&value, &mut canonical)?;
    Ok(sha256_hash(canonical.as_bytes()))
}

fn write_canonical_json(value: &Value, output: &mut String) -> Result<(), serde_json::Error> {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => output.push_str(&serde_json::to_string(value)?),
        Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(']');
        }
        Value::Object(values) => {
            output.push('{');
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key)?);
                output.push(':');
                write_canonical_json(value, output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}
