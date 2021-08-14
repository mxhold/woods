use bevy::prelude::*;

fn main() {
    App::build()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_startup_system(setup.system())
        .add_system_to_stage(CoreStage::PreUpdate, handle_messages_server.system())
        .add_system_to_stage(CoreStage::PostUpdate, network_broadcast_system.system())
        .run();
}
