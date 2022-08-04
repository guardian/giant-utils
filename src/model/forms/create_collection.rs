use serde::Serialize;

#[derive(Serialize)]
pub struct CreateCollection {
    pub name: String,
}
