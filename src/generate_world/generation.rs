use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use noise::{NoiseFn, OpenSimplex, Seedable};

const SCALE: f32 = 0.8;

pub(super) trait WorldGenerator {
    fn new(width: u32, height: u32) -> Self;
    fn get_elevation_map(&self, imgs: ResMut<Assets<Image>>) -> Handle<Image>;
}

#[derive(Clone, Debug, Default)]
pub(super) struct SimplexGenerator {
    width: u32,
    height: u32,
    gen: OpenSimplex,
}

impl PartialEq<SimplexGenerator> for SimplexGenerator {
    fn eq(&self, other: &SimplexGenerator) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.gen.seed() == other.gen.seed()
    }
}

impl Eq for SimplexGenerator {}

impl SimplexGenerator {
    fn get_map(&self) -> Vec<f32> {
        let mut image = Vec::with_capacity((self.width * self.height) as usize);

        for i in 0..self.width {
            let i = i as f32 / self.width as f32;
            for j in 0..self.height {
                let j = j as f32 / self.height as f32 * 4.0;

                let theta = (i - 0.5) * std::f32::consts::PI;
                let phi = j * std::f32::consts::TAU;

                let x = super::RADIUS * theta.cos() * phi.sin();
                let y = super::RADIUS * theta.sin() * phi.sin();
                let z = super::RADIUS * phi.cos();

                let mut acc = 0.0;
                let mut min_acc = 0.0;

                for harmonic in 0..8 {
                    let weight = 1. - harmonic as f32 / 8.;
                    let proj = (
                        x / (harmonic + 1) as f32,
                        y / (harmonic + 1) as f32,
                        z / (harmonic + 1) as f32,
                    );
                    acc +=
                        self.gen.get([proj.0 as f64, proj.1 as f64, proj.2 as f64]) as f32 * weight;
                    min_acc -= weight;
                }

                image.push(acc / min_acc * SCALE);
            }
        }

        image
    }

    fn get_bytes(inp: &[f32]) -> Vec<u8> {
        let mut out = Vec::with_capacity(inp.len() * 4);

        for f in inp {
            out.extend_from_slice(&f.to_le_bytes());
        }

        return out;
    }
}

impl WorldGenerator for SimplexGenerator {
    fn new(width: u32, height: u32) -> Self {
        SimplexGenerator {
            width,
            height,
            gen: OpenSimplex::new().set_seed(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time moved backwards")
                    .as_secs() as u32,
            ),
        }
    }

    fn get_elevation_map(&self, mut images: ResMut<Assets<Image>>) -> Handle<Image> {
        let img = self.get_map();
        let img = Self::get_bytes(&img[..]);
        let img = Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                ..default()
            },
            TextureDimension::D2,
            img,
            TextureFormat::R32Float,
        );

        images.add(img)
    }
}
