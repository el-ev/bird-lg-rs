#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModalState {
    pub active: bool,
    pub content: String,
    pub command: Option<String>,
}

pub enum ModalAction {
    Open {
        content: String,
        command: Option<String>,
    },
    UpdateContent(String),
    Close,
}

impl ModalState {
    pub fn reduce(&mut self, action: ModalAction) {
        match action {
            ModalAction::Open { content, command } => {
                self.active = true;
                self.content = content;
                self.command = command;
            }
            ModalAction::UpdateContent(content) => {
                self.content = content;
            }
            ModalAction::Close => {
                self.active = false;
                self.content = String::new();
                self.command = None;
            }
        }
    }
}
