mod module_bindings;
use module_bindings::*;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
    time::{self, Duration},
};

use spacetimedb_sdk::{DbContext, Table, credentials};

const VEHICLE_SPEED_PATH: &str = "Vehicle.Speed";
const PUBLISH_TIME_MILLIS: Duration = time::Duration::from_millis(1000);
const MAX_SPEED_KPH: f32 = 250.0;
const HOST: &str = "http://localhost:3000";
const DB_NAME: &str = "vssard";

fn creds_store() -> credentials::File {
    credentials::File::new("credentials.json")
}

/// Load credentials from a file and connect to the database.
fn connect_to_db(host: &String) -> DbConnection {
    let connection = DbConnection::builder()
        .with_token(creds_store().load().expect("Error loading credentials"))
        // Set the database name we chose when we called `spacetime publish`.
        .with_module_name(DB_NAME)
        // Set the URI of the SpacetimeDB host that's running our database.
        .with_uri(host)
        // Finalize configuration and connect!
        .build()
        .expect("Failed to connect");

    connection.run_threaded();
    connection
}

fn on_sub_applied(
    ctx: &SubscriptionEventContext,
    metadata: Arc<RwLock<Metadata>>,
    pair: Arc<(Mutex<bool>, Condvar)>,
) {
    let mut messages = ctx.db.metadata().iter().collect::<Vec<_>>();
    messages.sort_by_key(|m| m.id);

    let (mutex, cond) = &*pair;
    let mut guard = mutex.lock().unwrap();

    let mut metadata_lock = metadata.write().unwrap();

    for msg in messages {
        println!("Metadata update [{}]: {:?}", msg.path, msg.id);
        metadata_lock.add(msg.path, msg.id);
    }

    *guard = true;

    cond.notify_one();
}

fn publish_vehicle_speed(ctx: Rc<DbConnection>, vehicle_datapoint_id: u32, vehicle_speed: f32) {
    let set_result = ctx
        .reducers()
        .set_datapoint_value(vehicle_datapoint_id, VariantType::Float(vehicle_speed));

    if let Err(error) = set_result {
        println!("Error publishing vehicle speed: {}", error);
    }
}

struct Metadata {
    id_map: HashMap<String, u32>,
}

impl Metadata {
    fn new() -> Self {
        Metadata {
            id_map: HashMap::new(),
        }
    }

    fn add(&mut self, path: String, id: u32) {
        self.id_map.insert(path, id);
    }

    fn get(&self, path: &String) -> Option<u32> {
        self.id_map.get(path).cloned()
    }
}

struct CanClient {
    db_connection: Option<Rc<DbConnection>>,
    host: String,
    metadata: Arc<RwLock<Metadata>>,
    pair: Arc<(Mutex<bool>, Condvar)>,
}

impl CanClient {
    fn new(host: String) -> Self {
        CanClient {
            db_connection: None,
            host,
            metadata: Arc::new(RwLock::new(Metadata::new())),
            pair: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    fn connect(&mut self) -> Rc<DbConnection> {
        if let Some(ref db_connection) = self.db_connection {
            return db_connection.clone();
        }

        self.db_connection = Some(Rc::new(connect_to_db(&self.host)));
        if let Some(ref db_connection) = self.db_connection {
            db_connection.clone()
        } else {
            panic!("Failed to connect to the database");
        }
    }

    fn setup(&mut self) {
        let query = format!(
            "SELECT * FROM {} WHERE path = '{}'",
            "metadata", VEHICLE_SPEED_PATH
        );
        let connection = self.connect();

        let result = connection
            .reducers
            .register_datapoint(String::from(VEHICLE_SPEED_PATH));

        if let Err(error) = result {
            println!("Error registering vehicle speed datapoint: {}", error);
        } else {
            println!("Vehicle speed datapoint registered successfully");
        }

        let pair = self.pair.clone();
        let cloned_metadata = self.metadata.clone();
        connection
            .subscription_builder()
            .on_applied(|ctx| {
                on_sub_applied(ctx, cloned_metadata, pair);
            })
            .subscribe(query);
    }

    fn run(&mut self) {
        self.setup();

        let cloned_pair = self.pair.clone();
        let mut started = cloned_pair.0.lock().unwrap();
        // As long as the value inside the `Mutex<bool>` is `false`, we wait.
        while !*started {
            started = cloned_pair.1.wait(started).unwrap();
        }

        let mut vehicle_speed_id: Option<u32> = None;
        let key = String::from(VEHICLE_SPEED_PATH);

        if let Some(vehicle_speed_id_) = self.metadata.read().unwrap().get(&key) {
            vehicle_speed_id.replace(vehicle_speed_id_);
        }

        self.publish_loop(vehicle_speed_id.unwrap())
    }

    fn publish_loop(&mut self, id: u32) {
        let mut vehicle_speed = 0.0;

        loop {
            publish_vehicle_speed(self.connect().clone(), id, vehicle_speed);
            thread::sleep(PUBLISH_TIME_MILLIS);

            vehicle_speed += 1.0;
            if vehicle_speed > MAX_SPEED_KPH {
                vehicle_speed = 0.0;
            }
        }
    }
}

fn main() {
    let mut can_client = CanClient::new(String::from(HOST));
    can_client.connect();
    can_client.run();
}
