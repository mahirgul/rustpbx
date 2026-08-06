#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionKey {
    pub branch: String,
    pub cseq_method: String,
}

impl TransactionKey {
    pub fn new(branch: impl Into<String>, method: impl Into<String>) -> Self {
        TransactionKey {
            branch: branch.into(),
            cseq_method: method.into(),
        }
    }
}
