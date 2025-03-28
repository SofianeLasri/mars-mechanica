use bevy::asset::Handle;
use bevy::audio::AudioSource;
use bevy::image::Image;
use bevy::prelude::{Font, Resource};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct UiAssets {
    pub(crate) images: Vec<Handle<Image>>,
    pub(crate) sounds: Vec<Handle<AudioSource>>,
    pub(crate) fonts: Vec<Handle<Font>>,
}

#[derive(Resource, Default)]
pub struct TerrainAssets {
    pub materials: HashMap<String, HashMap<String, Handle<Image>>>,
}