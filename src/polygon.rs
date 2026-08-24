#![allow(clippy::identity_op)]

use crate::Vertex;
use std::ops::Range;
use std::{cmp::Ordering, mem};

// using TAU because 2PI *probably* has one bit less precision
use std::f32::consts::{PI, TAU};

use super::color;

#[derive(Debug, Clone)]
pub struct PolygonConfig<const MAX_CORNERS: usize, const MAX_INDECES: usize> {
    corners: u16,
    /// in radiance
    angle_offset: f32,
    corner_update_mode: CornerUpdateMode,
}

#[derive(Debug, Clone)]
pub enum CornerUpdateMode {
    Raw,
    Rotation { rotation: f32 },
    MultiSource { sources: u16 },
}

pub const fn index_count(corners: usize) -> usize {
    (corners - 2) * 3
}
impl<const MAX_CORNERS: usize, const MAX_INDECES: usize> PolygonConfig<MAX_CORNERS, MAX_INDECES> {
    pub const fn new(corners: u16) -> Self {
        let s = Self {
            corners,
            angle_offset: 0.0,
            corner_update_mode: CornerUpdateMode::Raw,
        };
        debug_assert!(s.corners <= MAX_CORNERS as _);
        debug_assert!(s.index_count() <= MAX_INDECES as _);
        s
    }
    pub const fn max_corners(&self) -> usize {
        MAX_CORNERS
    }
    pub const fn max_indeces(&self) -> usize {
        MAX_INDECES
    }

    pub fn mode_color(&self) -> [f32; 3] {
        match self.corner_update_mode {
            CornerUpdateMode::Raw => color::rgb_to_f32x3(0xCCCCCC),
            CornerUpdateMode::Rotation { .. } => color::rgb_to_f32x3(0x0000FF),
            CornerUpdateMode::MultiSource { .. } => color::rgb_to_f32x3(0xFFFF00),
        }
    }

    pub const fn vertex_count_bytes(&self) -> u64 {
        (self.corners as usize * mem::size_of::<Vertex>()) as _
    }
    pub const fn write_vertices(&self, vertices: &mut [Vertex; MAX_CORNERS]) {
        use trig_const::{cos, sin};

        let mut idx = 0;
        while idx < self.corners {
            let angle = self.angle_size() * idx as f32 + self.angle_offset;
            let pos = &mut vertices[idx as usize].position;
            pos[0] = sin(angle as _) as _;
            pos[1] = cos(angle as _) as _;

            idx += 1;
        }
    }
    pub const fn angle_size(&self) -> f32 {
        TAU / self.corners as f32
    }

    pub const fn index_count(&self) -> u32 {
        ((self.corners - 2) * 3) as _
    }
    pub const fn index_count_bytes(&self) -> u64 {
        self.index_count() as u64 * mem::size_of::<u16>() as u64
    }
    pub const fn write_indeces_ccw(&self, out: &mut [u16; MAX_INDECES]) {
        const fn set_tri(out: &mut &mut [u16], a: u16, b: u16, c: u16) {
            out[0] = a;
            out[1] = b;
            out[2] = c;

            #[allow(clippy::mem_replace_with_default)]
            match mem::replace(out, &mut []) {
                [out_a, out_b, out_c, rest @ ..] => {
                    *out_a = a;
                    *out_b = b;
                    *out_c = c;
                    *out = rest;
                }
                _ => unreachable!(),
            }
        }

        {
            let mut out = out.as_mut_slice();
            let mut idx = 2u16;
            while idx < self.corners {
                set_tri(&mut out, idx, idx - 1, 0);
                idx += 1;
            }
        }
    }

    pub fn update_corners(&mut self, num: i16) {
        match self.corner_update_mode {
            CornerUpdateMode::Raw => self.update_corners_raw(num),
            CornerUpdateMode::Rotation { rotation } => self.update_corners_rotation(num, rotation),
            CornerUpdateMode::MultiSource { sources } => self.update_corners_multi(num, sources),
        }
    }
    pub fn update_next_corner_mode(&mut self) {
        self.corner_update_mode = match self.corner_update_mode {
            CornerUpdateMode::Raw => CornerUpdateMode::Rotation { rotation: 1. },
            CornerUpdateMode::Rotation { .. } => CornerUpdateMode::MultiSource { sources: 2 },
            CornerUpdateMode::MultiSource { .. } => CornerUpdateMode::Raw,
        };
    }
    pub fn get_corner_mode_mut(&mut self) -> &mut CornerUpdateMode {
        &mut self.corner_update_mode
    }
    fn update_corners_with(&mut self, num: i16) -> (f32, Range<u16>) {
        let old = self.corners;
        self.update_corners_raw(num);
        let new = self.corners;
        match old.cmp(&new) {
            Ordering::Less => (0.0 + 1., old..new),
            Ordering::Equal => (0.0, old..new),
            Ordering::Greater => (0. - 1., new..old),
        }
    }
    pub fn update_corners_raw(&mut self, num: i16) {
        self.corners = self
            .corners
            .checked_add_signed(num)
            .unwrap_or_else(|| match 0.cmp(&num) {
                Ordering::Less | Ordering::Equal => 3,
                Ordering::Greater => MAX_CORNERS as _,
            })
            .clamp(3, MAX_CORNERS as _);
    }
    pub fn update_corners_rotation(&mut self, num: i16, rotation: f32) {
        let (sign, range) = self.update_corners_with(num);
        let sum: f32 = range.into_iter().map(|val| 1. / val as f32).sum();
        let summed_angle_change = sign * sum * TAU * rotation;
        self.update_angle(summed_angle_change);
    }
    fn update_corners_multi(&mut self, num: i16, sources: u16) {
        let (sign, range) = self.update_corners_with(num);

        let pos = |val: u16| {
            let val = val as f32;
            let sources = sources as f32;
            let res = angle_to_pos(val / sources * TAU) / val / 2.;
            log::info!("{val}/{sources} * {TAU} = {res}");
            res
        };
        let delta = sign * range.into_iter().map(pos).sum::<f32>() * TAU;
        self.update_angle(delta);
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.angle_offset = angle.rem_euclid(TAU);
    }
    pub fn update_angle(&mut self, angle: f32) {
        self.angle_offset = (self.angle_offset + angle).rem_euclid(TAU);
    }
}

fn angle_to_pos(angle: f32) -> f32 {
    (angle.rem_euclid(TAU) - PI) * 2. / TAU
}
