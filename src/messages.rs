use serde::{Serialize, Deserialize};

use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct LogInfo {
    message: String,
    extra: String,
}

impl LogInfo {
    pub fn new_random() -> Self {
        let id = Uuid::new_v4();

        LogInfo {
            message: "New message to process.".into(),
            extra: format!("The unique id is: {}", id)
        }
    }
}
