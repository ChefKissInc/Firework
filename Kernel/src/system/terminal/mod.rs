// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::fmt::Write;

use amd64::paging::pml4::PML4;
use paper_fb::{fb::FrameBuffer, pixel::Colour};

mod font;

pub struct Terminal {
    pub x: usize,
    pub y: usize,
    pub fb: FrameBuffer,
    pub width: usize,
    pub height: usize,
}

unsafe impl Send for Terminal {}
unsafe impl Sync for Terminal {}

impl Terminal {
    #[inline]
    pub const fn new(fb: FrameBuffer) -> Self {
        let width = fb.width / 8;
        let height = fb.height / 16;
        Self {
            x: 0,
            y: 0,
            fb,
            width,
            height,
        }
    }

    pub fn map_fb(&self) {
        unsafe {
            let state = &mut *super::state::SYS_STATE.get();
            let base = self.fb.base.as_ptr() as u64;
            state.pml4.as_mut().unwrap().map_huge_pages(
                base,
                base - amd64::paging::PHYS_VIRT_OFFSET,
                ((self.fb.height * self.fb.stride + 0x1F_FFFF) / 0x20_0000) as _,
                amd64::paging::PageTableEntry::new()
                    .with_writable(true)
                    .with_present(true)
                    .with_pcd(true),
            );
        }
    }

    pub fn clear(&mut self) {
        self.fb.clear(0);
        self.x = 0;
        self.y = 0;
    }

    pub fn draw_char(&mut self, c: char, colour: Colour) {
        let x = self.x * 8;
        let mut y = self.y * 16;
        let Some(v) = font::FONT_BITMAP.get(c as usize - 0x21) else {
            return;
        };
        for &x_bit in v {
            for bit in 0..8 {
                if x_bit & (1 << bit) != 0 {
                    self.fb
                        .plot_pixel(x + 8 - bit, y, colour.as_u32(self.fb.bitmask))
                        .unwrap();
                }
            }
            y += 1;
        }
    }

    fn handle_scrollback(&mut self) {
        if self.y >= self.height {
            self.fb
                .base
                .copy_within(self.fb.stride * 16..self.fb.stride * self.fb.height, 0);
            self.fb.base[self.fb.stride * (self.fb.height - 16)..].fill(0);
            self.y -= 1;
            self.x = 0;
        }
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.y += 1;
                self.x = 0;
                self.handle_scrollback();
            } else {
                self.draw_char(c, Colour::new(0xFF, 0xFF, 0xFF, 0xFF));
                self.x += 1;
                if self.x >= self.width {
                    self.y += 1;
                    self.x = 0;
                    self.handle_scrollback();
                }
            }
        }
        Ok(())
    }
}