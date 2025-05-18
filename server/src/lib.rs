use spacetimedb::{reducer, ReducerContext, SpacetimeType, Table};

#[derive(SpacetimeType, Debug)]
pub enum VariantType {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
}

// Datapoint table to store the current datapoint values
#[spacetimedb::table(name = datapoint, public)]
pub struct Datapoint {
    #[primary_key]
    id: u32,
    value: VariantType,
    target_value: Option<VariantType>,
}

// Metadata table to store metadata about the datapoints
// Stores the relation from path to id
#[spacetimedb::table(name = metadata, public)]
pub struct Metadata {
    #[primary_key]
    path: String,
    id: u32,
}

#[reducer]
pub fn register_datapoint(ctx: &ReducerContext, path: String) -> Result<(), String> {
    if let None = ctx.db.metadata().path().find(&path) {
        ctx.db.metadata().insert(Metadata {
            path: path.clone(),
            id: ctx.db.metadata().count() as u32,
        });
        Ok(())
    } else {
        Err(format!("Datapoint {} already registered!", path))
    }
}

#[reducer]
pub fn set_datapoint_value(
    ctx: &ReducerContext,
    id: u32,
    value: VariantType,
) -> Result<(), String> {
    log::error!("Setting value of datapoint {} to {:?}", id, value);
    if let Some(datapoint) = ctx.db.datapoint().id().find(id) {
        ctx.db
            .datapoint()
            .id()
            .update(Datapoint { value, ..datapoint });
        Ok(())
    } else {
        ctx.db.datapoint().insert(Datapoint {
            id,
            value,
            target_value: None,
        });
        Ok(())
    }
}

#[reducer]
pub fn set_datapoint_target_value(
    ctx: &ReducerContext,
    id: u32,
    target_value: VariantType,
) -> Result<(), String> {
    if let Some(datapoint) = ctx.db.datapoint().id().find(id) {
        ctx.db.datapoint().id().update(Datapoint {
            target_value: Some(target_value),
            ..datapoint
        });
        Ok(())
    } else {
        Err("Cannot set the desired value of an unknown datapoint!".to_string())
    }
}
