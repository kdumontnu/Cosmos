//! Represents various structure events

use bevy::prelude::{App, Entity};

use super::{structure_iterator::BlockIterator, Structure};

/// This will be created once all chunks have been populated
pub struct StructureLoadedEvent {
    /// The entity that contains this structure - make sure this entity is still valid before using!
    pub structure_entity: Entity,
}

/// This should only be used to initially setup a structure.
/// Do **not** overwrite existing blocks with this.
/// Some systems will get out of sync if you misuse this.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ChunkSetEvent {
    /// The entity of the structure this is a part of - make sure this is valid before using!
    pub structure_entity: Entity,
    /// Chunk's coordinate in the structure
    pub x: usize,
    /// Chunk's coordinate in the structure    
    pub y: usize,
    /// Chunk's coordinate in the structure    
    pub z: usize,
}

impl ChunkSetEvent {
    /// Iterates over all the blocks of this structure.
    ///
    /// * `include_air` If this is true, air blocks will be included. If false, they will not be
    pub fn iter_blocks<'a>(&'a self, structure: &'a Structure, include_air: bool) -> BlockIterator {
        structure.block_iter_for_chunk((self.x, self.y, self.z), include_air)
    }
}

pub(super) fn register(app: &mut App) {
    app.add_event::<ChunkSetEvent>().add_event::<StructureLoadedEvent>();
}
