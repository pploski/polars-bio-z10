use std::sync::Arc;
use arrow_array::{Array, StringArray};
use arrow_schema::DataType;
use datafusion::common::{DataFusionError, Result, ScalarValue};
use datafusion::logical_expr::{create_udf, ColumnarValue, ScalarUDF, Volatility};

pub fn create_gc_calculate_udf() -> ScalarUDF {
    let func = |args: &[ColumnarValue]| -> Result<ColumnarValue> {
        let array = match &args[0] {
            ColumnarValue::Array(arr) => arr
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| DataFusionError::Execution("Expected StringArray".to_string()))?,
            _ => return Err(DataFusionError::Execution("Expected array input".to_string())),
        };

        let mut output = Vec::with_capacity(array.len());

        for i in 0..array.len() {
            if array.is_null(i) {
                output.push(ScalarValue::Float64(None));
                continue;
            }

            let bytes = array.value(i).as_bytes();
            let (mut total, mut gc) = (0, 0);
            for &b in bytes {
                if !b.is_ascii_whitespace() {
                    total += 1;
                    if matches!(b, b'G' | b'g' | b'C' | b'c') {
                        gc += 1;
                    }
                }
            }

            let gc_percent = if total > 0 {
                (gc as f64) / (total as f64) * 100.0
            } else {
                0.0
            };

            output.push(ScalarValue::Float64(Some(gc_percent)));
        }

        Ok(ColumnarValue::Array(ScalarValue::iter_to_array(output)?))
    };

    create_udf(
        "my_gc_calculate",
        vec![DataType::Utf8],
        DataType::Float64,
        Volatility::Immutable,
        Arc::new(func),
    )
}






