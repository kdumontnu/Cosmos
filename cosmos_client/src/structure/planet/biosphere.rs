//! Represents client information for biospheres

use bevy::prelude::{App, Color, ResMut, Startup};
use cosmos_core::registry::{self, identifiable::Identifiable, Registry};

#[derive(Debug, Clone)]
/// Represents the overall color of a biosphere
pub struct BiosphereColor {
    color: Color,
    id: u16,
    unlocalized_name: String,
}

impl BiosphereColor {
    /// Creates a color entry for this biosphere's unlocalized name
    pub fn new(name: impl Into<String>, color: Color) -> Self {
        Self {
            color,
            id: 0,
            unlocalized_name: name.into(),
        }
    }

    /// Gets the color for this biosphere
    pub fn color(&self) -> Color {
        self.color
    }
}

impl Identifiable for BiosphereColor {
    fn id(&self) -> u16 {
        self.id
    }

    fn unlocalized_name(&self) -> &str {
        &self.unlocalized_name
    }

    fn set_numeric_id(&mut self, id: u16) {
        self.id = id
    }
}

fn register_biospheres(mut reigstry: ResMut<Registry<BiosphereColor>>) {
    reigstry.register(BiosphereColor::new("cosmos:biosphere_grass", Color::GREEN));
    reigstry.register(BiosphereColor::new("cosmos:biosphere_molten", Color::ORANGE_RED));
    reigstry.register(BiosphereColor::new("cosmos:biosphere_ice", Color::CYAN));
}

pub(super) fn register(app: &mut App) {
    registry::create_registry::<BiosphereColor>(app, "cosmos:biosphere_colors");

    app.add_systems(Startup, register_biospheres);
}
