#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Calling,
    Trying,
    Proceeding,
    Completed,
    Confirmed,
    Terminated,
}
