#![allow(clippy::identity_op)]

use crate::Vertex;
use std::{cmp::Ordering, mem};

// using TAU because 2PI *probably* has one bit less precision
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct PolygonConfig<const MAX_CORNERS: usize, const MAX_INDECES: usize> {
    corners: u16,
    /// in radiance
    angle_offset: f32,
}

pub const fn index_count(corners: usize) -> usize {
    (corners - 2) * 3
}
impl<const MAX_CORNERS: usize, const MAX_INDECES: usize> PolygonConfig<MAX_CORNERS, MAX_INDECES> {
    pub const fn new(corners: u16) -> Self {
        let s = Self {
            corners,
            angle_offset: 0.0,
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

    pub const fn vertex_count_bytes(&self) -> u64 {
        (self.corners as usize * mem::size_of::<Vertex>()) as _
    }
    pub const fn write_vertices(&self, vertices: &mut [Vertex; MAX_CORNERS]) {
        use trig_const::{cos, sin};

        let angle = TAU / self.corners as f32;

        let mut idx = 0;
        while idx < self.corners {
            let angle = angle * idx as f32 + self.angle_offset;
            let pos = &mut vertices[idx as usize].position;
            pos[0] = sin(angle as _) as _;
            pos[1] = cos(angle as _) as _;

            idx += 1;
        }
    }
    pub fn angle_size(&self) -> f32 {
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
        self.corners = self
            .corners
            .checked_add_signed(num)
            .unwrap_or_else(|| match 0.cmp(&num) {
                Ordering::Less | Ordering::Equal => 3,
                Ordering::Greater => MAX_CORNERS as _,
            })
            .clamp(3, MAX_CORNERS as _);
    }
    pub fn update_corners_smooth(&mut self, num: i16) {
        let old = self.corners;
        self.update_corners(num);
        let new = self.corners;
        let (sign, range) = match old.cmp(&new) {
            Ordering::Less => (0.0 + 1., old..new),
            Ordering::Equal => (0.0, old..new),
            Ordering::Greater => (0. - 1., new..old),
        };
        //   1/3 + 1/4 + 1/5
        // = 4*5/3*4*5 + 3*5/3*4*5 + 3*4/3*4*5
        let product = range
            .clone()
            .into_iter()
            .map(|val| val as f32)
            .product::<f32>();
        let sum: f32 = range.into_iter().map(|val| product / val as f32).sum();

        let summed_angle_change = sign * TAU / product * sum;
        self.update_angle(summed_angle_change);
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.angle_offset = angle.rem_euclid(TAU);
    }
    pub fn update_angle(&mut self, angle: f32) {
        self.angle_offset = (self.angle_offset + angle).rem_euclid(TAU);
    }
}
