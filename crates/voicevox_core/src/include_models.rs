[
    Model{
        yukarin_s_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/yukarin_s.onnx"
    )),
        yukarin_sa_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/yukarin_sa.onnx"
    )),
        decode_model: include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"), 
        "/model/decode.onnx"
    )),
    },
]
