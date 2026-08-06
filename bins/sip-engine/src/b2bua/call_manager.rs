use crate::b2bua::call_bridge::CallBridge;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

pub struct CallManager {
    pub calls: DashMap<String, Arc<CallBridge>>,
}

impl Default for CallManager {
    fn default() -> Self {
        CallManager {
            calls: DashMap::new(),
        }
    }
}

impl CallManager {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn insert_call(&self, bridge: CallBridge) {
        info!("Registering active call bridge ID: {}", bridge.call_id);
        self.calls.insert(bridge.call_id.clone(), Arc::new(bridge));
    }

    pub fn remove_call(&self, call_id: &str) -> Option<Arc<CallBridge>> {
        info!("Removing call bridge ID: {}", call_id);
        self.calls.remove(call_id).map(|(_, b)| b)
    }

    pub fn active_call_count(&self) -> usize {
        self.calls.len()
    }
}
