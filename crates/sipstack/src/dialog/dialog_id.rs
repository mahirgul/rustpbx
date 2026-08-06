#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogId {
    pub call_id: String,
    pub local_tag: String,
    pub remote_tag: String,
}

impl DialogId {
    pub fn new(
        call_id: impl Into<String>,
        local_tag: impl Into<String>,
        remote_tag: impl Into<String>,
    ) -> Self {
        DialogId {
            call_id: call_id.into(),
            local_tag: local_tag.into(),
            remote_tag: remote_tag.into(),
        }
    }
}
