use bevy::prelude::*;

pub(super) trait WorldGenerator {
    fn new(width: u32, height: u32) -> Self;
    fn get_elevation_map(&self) -> Handle<Image>;
}

#[derive(Clone, Debug, Hash, PartialEq, Default)]
pub(super) struct SimplexGenerator;
