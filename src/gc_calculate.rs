use std::sync::Arc;
use std::thread::spawn;

use datafusion::arrow::datatypes::DataType;
use datafusion::execution::context::SessionContext;
use datafusion::physical_plan::functions::{make_scalar_function, ReturnTypeFunction, Volatility};
use datafusion::scalar::ScalarValue;
use exon::ExonSession;

fn compute_gc_content(seq: &str) -> f64 {
    let gc = seq.chars().filter(|c| *c == 'G' || *c == 'C').count();
    gc as f64 / seq.len().max(1) as f64
}

pub fn register_gc_content_udf(ctx: &ExonSession) {
    let gc_func = make_scalar_function(|args: &[ScalarValue]| {
        if let ScalarValue::Utf8(Some(seq)) = &args[0] {
            let gc = compute_gc_content(seq);
            Ok(ScalarValue::Float64(Some(gc)))
        } else {
            Ok(ScalarValue::Float64(None))
        }
    });

    let return_type: ReturnTypeFunction = Arc::new(|_| Ok(Arc::new(DataType::Float64)));

    let udf = datafusion::physical_plan::functions::create_udf(
        "gc_content",
        vec![DataType::Utf8],
        Arc::new(DataType::Float64),
        Volatility::Immutable,
        gc_func,
    );

    ctx.session.register_udf(udf);
}
