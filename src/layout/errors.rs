#[derive(Debug)]
pub enum LayoutDbError {
    CellNameAlreadyExists(String),
    CellNameNotFound(String),
    CellIndexNotFound
}