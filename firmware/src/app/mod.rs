// SPDX-License-Identifier: Apache-2.0

//! Composition and task ownership. Domain state stays local to its owner.
pub mod messages;
mod tasks;

use analyzer_core::state::RadioState;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use crate::{board::Resources, platform::radio::Radio};

pub async fn run(spawner: Spawner, resources: Resources) -> ! {
    let radio_state = match Radio::new(resources.wifi) {
        Ok(radio) => {
            spawner.spawn(tasks::radio_worker(radio).expect("allocate radio worker task"));
            RadioState::Initialized
        }
        Err(_) => RadioState::Failed,
    };
    // Keep the CLI operational even if radio initialization fails.
    spawner.spawn(tasks::analyzer(radio_state).expect("allocate analyzer task"));
    spawner.spawn(tasks::console(resources.console).expect("allocate console task"));
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
