
#[derive(Clone)]
pub struct WizardData {
    pub name: String,
    pub email: String,
    pub description: String,
    pub current_step: usize,
    pub attachment: Option<(String, Vec<u8>)>, // (filename, data)
    pub message: String,
}

impl Default for WizardData {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            description: String::new(),
            current_step: 0,
            attachment: None,
            message: String::new(),
        }
    }
}

