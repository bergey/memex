use super::RecordId;

pub enum Action {
    SetName(String),
    AddRecord(),
    SetTitle(RecordId, String),
    DeleteRecord(RecordId)
}
