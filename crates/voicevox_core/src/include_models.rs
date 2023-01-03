[
    Model{
        predict_duration_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/predict_duration-0.onnx"
    )),
        predict_intonation_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/predict_intonation-0.onnx"
    )),
        decode_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"), 
        "/model/decode-0.onnx"
    )),
    },
    Model{
        predict_duration_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/predict_duration-1.onnx"
    )),
        predict_intonation_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/predict_intonation-1.onnx"
    )),
        decode_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"), 
        "/model/decode-1.onnx"
    )),
    },
]
