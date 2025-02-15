pub trait ToJsonValue {
    fn to_json_value(&self) -> serde_json::Value;

    fn to_json(&self) -> String {
        self.to_json_value().to_string()
    }
}
