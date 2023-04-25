//! Contains server-side logic for the universe & how it's generated

use bevy::prelude::App;

pub mod generation;
pub mod planet_spawner;
pub mod star;

pub(super) fn register(app: &mut App) {
    star::register(app);
    generation::register(app);
    planet_spawner::register(app);
}
