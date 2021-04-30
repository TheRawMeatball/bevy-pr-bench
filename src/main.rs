use std::time::Duration;

use bevy::{app::AppExit, asset::LoadState, log::LogPlugin, prelude::*, winit::WinitConfig};
use crossbeam_channel::Sender;

struct Handles(Vec<HandleUntyped>);

const RUN_COUNT: usize = 10;

fn main() {
    let (tx, rx) = crossbeam_channel::bounded::<Duration>(RUN_COUNT);
    for i in 0..RUN_COUNT {
        App::build()
            .add_plugins_with(DefaultPlugins, |builder| {
                if i != 0 {
                    builder.disable::<LogPlugin>()
                } else {
                    builder
                }
            })
            .insert_resource(tx.clone())
            .insert_resource(WinitConfig {
                return_from_run: true,
            })
            .add_startup_system(
                (|assets: Res<AssetServer>, mut commands: Commands| {
                    let assets = assets.load_folder("").unwrap();
                    commands.insert_resource(Handles(assets));
                })
                .system(),
            )
            .add_system(
                (|assets: Res<AssetServer>,
                  handles: Res<Handles>,
                  time: Res<Time>,
                  tx: Res<Sender<Duration>>,
                  mut exit: EventWriter<AppExit>| match assets
                    .get_group_load_state(handles.0.iter().map(|h| h.id))
                {
                    LoadState::Loaded => {
                        let tss = time.time_since_startup();
                        println!("done @ {:?}", tss);
                        tx.send(tss).unwrap();
                        exit.send(AppExit);
                    }
                    _ => {}
                })
                .system(),
            )
            .run();
    }
    let total: Duration = rx.try_iter().sum();
    let avg = total / RUN_COUNT as u32;
    println!("avg = {:?}", avg);
}
