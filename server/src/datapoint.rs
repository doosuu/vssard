use spacetimedb::{ReducerContext, SpacetimeType, Table};

#[derive(SpacetimeType, Debug)]
pub enum Datatype {
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
    value: Datatype,
    target_value: Option<Datatype>,
}

#[spacetimedb::reducer]
pub fn set_datapoint_value(
    ctx: &ReducerContext,
    id: u32,
    value: Datatype,
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

#[spacetimedb::reducer]
pub fn set_datapoint_target_value(
    ctx: &ReducerContext,
    id: u32,
    target_value: Datatype,
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
