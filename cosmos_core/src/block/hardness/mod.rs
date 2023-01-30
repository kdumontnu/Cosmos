use bevy::{
    ecs::schedule::StateData,
    prelude::{App, Res, ResMut, SystemSet},
};

use crate::registry::{self, identifiable::Identifiable, Registry};

use super::Block;

#[derive(Debug)]
pub struct BlockHardness {
    id: u16,
    unlocalized_name: String,

    // Air: 0, Leaves: 1, Grass/Dirt: 10, Stone: 50, Hull: 100,
    hardness: f32,
}

impl BlockHardness {
    pub fn new(block: &Block, hardness: f32) -> BlockHardness {
        Self {
            id: 0,
            unlocalized_name: block.unlocalized_name.to_owned(),
            hardness,
        }
    }

    pub fn hardness(&self) -> f32 {
        self.hardness
    }
}

impl Identifiable for BlockHardness {
    fn id(&self) -> u16 {
        self.id
    }

    fn set_numeric_id(&mut self, id: u16) {
        self.id = id;
    }

    fn unlocalized_name(&self) -> &str {
        &self.unlocalized_name
    }
}

fn register_hardness(
    registry: &mut Registry<BlockHardness>,
    value: f32,
    blocks: &Registry<Block>,
    name: &str,
) {
    if let Some(block) = blocks.from_id(name) {
        registry.register(BlockHardness::new(block, value));
    } else {
        println!("[Block Hardness] Missing block {name}");
    }
}

fn register_block_hardness(
    blocks: Res<Registry<Block>>,
    mut registry: ResMut<Registry<BlockHardness>>,
) {
    register_hardness(&mut registry, 0.0, &blocks, "cosmos:air");
    register_hardness(&mut registry, 10.0, &blocks, "cosmos:grass");
    register_hardness(&mut registry, 10.0, &blocks, "cosmos:dirt");
    register_hardness(&mut registry, 50.0, &blocks, "cosmos:stone");

    register_hardness(&mut registry, 30.0, &blocks, "cosmos:cherry_log");
    register_hardness(&mut registry, 1.0, &blocks, "cosmos:cherry_leaf");

    register_hardness(&mut registry, 100.0, &blocks, "cosmos:ship_core");
    register_hardness(&mut registry, 20.0, &blocks, "cosmos:energy_cell");
    register_hardness(&mut registry, 20.0, &blocks, "cosmos:reactor");
    register_hardness(&mut registry, 20.0, &blocks, "cosmos:laser_cannon");
    register_hardness(&mut registry, 20.0, &blocks, "cosmos:thruster");

    register_hardness(&mut registry, 100.0, &blocks, "cosmos:ship_hull");
}

fn sanity_check(blocks: Res<Registry<Block>>, hardness: Res<Registry<BlockHardness>>) {
    for block in blocks.iter() {
        if hardness.from_id(block.unlocalized_name()).is_none() {
            eprintln!(
                "!!! WARNING !!! Missing block hardness value for {}",
                block.unlocalized_name()
            );
        }
    }
}

pub(crate) fn register<T: StateData + Clone + Copy>(
    app: &mut App,
    pre_loading_state: T,
    loading_state: T,
    post_loading_state: T,
) {
    registry::register::<T, BlockHardness>(app, pre_loading_state);

    app.add_system_set(SystemSet::on_exit(loading_state).with_system(register_block_hardness))
        .add_system_set(SystemSet::on_exit(post_loading_state).with_system(sanity_check));
}
