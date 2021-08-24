use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use log::LevelFilter;
use network::NetworkPlugin;
use simple_logger::SimpleLogger;

mod network;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("woods_server", LevelFilter::Trace)
        .init()
        .unwrap();

    App::build()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(NetworkPlugin)
        .run();
}
