pub mod register;
pub mod search;
pub mod update;
pub mod preferences;
pub mod manage_tags;

pub use search::Search;
pub use register::Register;
pub use update::Update;
pub use preferences::Preferences;
pub use manage_tags::ManageTags;

pub enum Screen {
    Search(Search),
    Register(Register),
    Update(Update),
    Preferences(Preferences),
    ManageTags(ManageTags),
}
