use crate::dialog::dialog_id::DialogId;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogRole {
    Uas,
    Uac,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogState {
    Early,
    Confirmed,
    Terminated,
}

pub struct InviteDialog {
    pub id: DialogId,
    pub role: DialogRole,
    pub state: DialogState,
    pub local_cseq: Arc<AtomicU32>,
    pub remote_cseq: Arc<AtomicU32>,
}

impl InviteDialog {
    pub fn new(id: DialogId, role: DialogRole) -> Self {
        InviteDialog {
            id,
            role,
            state: DialogState::Early,
            local_cseq: Arc::new(AtomicU32::new(1)),
            remote_cseq: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn next_local_cseq(&self) -> u32 {
        self.local_cseq.fetch_add(1, Ordering::SeqCst)
    }
}
