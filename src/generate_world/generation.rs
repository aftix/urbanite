use std::{
    future::Future,
    pin::Pin,
    sync::{mpsc::channel, Mutex},
    task::{Context, Poll},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use noise::{NoiseFn, OpenSimplex, Seedable};

const SCALE: f32 = 0.8;

pub(super) trait WorldGenerator {
    fn new(width: u32, height: u32) -> Self;
    fn get_elevation_map(&self) -> Image;
}

pub(crate) struct GenerationTask<T> {
    generator: T,
    sender: std::sync::mpsc::Sender<Image>,
    receiver: std::sync::mpsc::Receiver<Image>,
    spawned: Mutex<bool>,
}

impl<T> GenerationTask<T> {
    pub fn new(generator: T) -> Self {
        let (tx, rx) = channel();
        Self {
            generator,
            sender: tx,
            receiver: rx,
            spawned: Mutex::new(false),
        }
    }
}

impl<T: WorldGenerator + Send + Clone + 'static> Future for GenerationTask<T> {
    type Output = Option<Image>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = self.receiver.try_recv();
        if let Ok(img) = res {
            return Poll::Ready(Some(img));
        }

        if let Err(std::sync::mpsc::TryRecvError::Disconnected) = res {
            return Poll::Ready(None);
        }

        let mut spawned = self.spawned.lock().expect("Lock failed");

        if !*spawned {
            let tx = self.sender.clone();
            let waker = ctx.waker().clone();
            let gen = self.generator.clone();
            std::thread::spawn(move || {
                tx.send(gen.get_elevation_map())
                    .expect("Failed to send elevation map");
                waker.wake();
            });
            *spawned = true;
        }

        Poll::Pending
    }
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

        out
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

    fn get_elevation_map(&self) -> Image {
        let img = self.get_map();
        let img = Self::get_bytes(&img[..]);
        Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                ..default()
            },
            TextureDimension::D2,
            img,
            TextureFormat::R32Float,
        )
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn is_simplexgenerator_sized() {
        use super::{SimplexGenerator, WorldGenerator};

        let gen = SimplexGenerator::new(500, 500);
        let map = gen.get_map();
        assert_eq!(map.len(), 500 * 500);

        let gen = SimplexGenerator::new(200, 150);
        let map = gen.get_map();
        assert_eq!(map.len(), 200 * 150);
    }

    #[test]
    fn is_simplexgenerator_clamped() {
        use super::{SimplexGenerator, WorldGenerator, SCALE};

        let gen = SimplexGenerator::new(200, 150);
        let map = gen.get_map();

        for f in map {
            assert!(f <= SCALE);
            assert!(f >= -SCALE);
        }
    }

    #[test]
    fn is_simplexgenerator_worldgenerator() {
        use super::{SimplexGenerator, WorldGenerator};
        use bevy::prelude::*;

        let gen = SimplexGenerator::new(500, 500);
        let img = gen.get_elevation_map();
        assert_eq!(img.size(), Vec2::new(500., 500.));
    }
}
