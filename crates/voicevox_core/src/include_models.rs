[
    Model{
        predict_duration_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/predict_duration.onnx"
    )),
        predict_intonation_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/predict_intonation.onnx"
    )),
        decode_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"), 
        "/model/decode.onnx"
    )),
    },
]
