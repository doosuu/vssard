use spacetimedb::{ReducerContext, Table};

// Metadata table to store metadata about the datapoints
// Stores the relation from path to id
#[spacetimedb::table(name = metadata, public)]
pub struct Metadata {
    #[primary_key]
    pub path: String,
    pub type_: String,
    pub description: String,
    pub datatype: String,
    id: u32,
}

impl Metadata {
    pub fn new(
        path: String,
        type_: String,
        description: String,
        datatype: String,
        id: u32,
    ) -> Self {
        Metadata {
            path,
            type_,
            description,
            datatype,
            id,
        }
    }
}

#[spacetimedb::reducer]
pub fn register_datapoint(ctx: &ReducerContext, mut metadata: Metadata) -> Result<(), String> {
    log::error!("Trying to register datapoint...");
    if let None = ctx.db.metadata().path().find(&metadata.path) {
        metadata.id = ctx.db.metadata().count() as u32;
        log::error!("Registed datapoint {} with ID {}", metadata.path, metadata.id);
        ctx.db.metadata().insert(metadata);
        Ok(())
    } else {
        Err(format!("Datapoint {} already registered!", metadata.path))
    }
}
