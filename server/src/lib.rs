
use spacetimedb::ReducerContext;
use vss::load_vehicle_signals;

mod datapoint;
mod metadata;
mod vss;


#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    load_vehicle_signals(ctx);
}
