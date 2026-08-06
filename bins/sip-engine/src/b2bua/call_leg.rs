use sipstack::dialog::{DialogRole, InviteDialog};

#[allow(dead_code)]
pub struct CallLeg {
    pub dialog: InviteDialog,
    pub target_uri: String,
}

#[allow(dead_code)]
impl CallLeg {
    pub fn new(dialog: InviteDialog, target_uri: impl Into<String>) -> Self {
        CallLeg {
            dialog,
            target_uri: target_uri.into(),
        }
    }

    pub fn role(&self) -> DialogRole {
        self.dialog.role
    }
}
