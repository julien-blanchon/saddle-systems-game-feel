use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::GameFeelPlugin;

#[test]
fn always_on_constructor_uses_requested_update_schedule() {
    let plugin = GameFeelPlugin::always_on(Update);
    assert_eq!(plugin.update_schedule, Update.intern());
}
