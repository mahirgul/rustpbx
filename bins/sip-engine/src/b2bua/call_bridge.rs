use crate::b2bua::call_leg::CallLeg;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CallState {
    Initiating,
    Ringing,
    Connected,
    Terminated,
}

#[allow(dead_code)]
pub struct CallBridge {
    pub call_id: String,
    pub leg_a: CallLeg, // UAS (incoming leg)
    pub leg_b: CallLeg, // UAC (outgoing leg)
    pub state: CallState,
    pub started_at: Instant,
    pub answered_at: Option<Instant>,
}

#[allow(dead_code)]
impl CallBridge {
    pub fn new(call_id: impl Into<String>, leg_a: CallLeg, leg_b: CallLeg) -> Self {
        CallBridge {
            call_id: call_id.into(),
            leg_a,
            leg_b,
            state: CallState::Initiating,
            started_at: Instant::now(),
            answered_at: None,
        }
    }

    pub fn mark_answered(&mut self) {
        self.state = CallState::Connected;
        self.answered_at = Some(Instant::now());
    }

    pub fn duration_secs(&self) -> u64 {
        match self.answered_at {
            Some(t) => t.elapsed().as_secs(),
            None => 0,
        }
    }
}
