// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::missing_safety_doc)]

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

extern crate alloc;

pub mod dt;
#[cfg(target_arch = "x86_64")]
pub mod port;
#[cfg(target_arch = "x86_64")]
pub mod syscall;

use serde::{Deserialize, Serialize};

pub const USER_PHYS_VIRT_OFFSET: u64 = 0xC000_0000;

#[derive(Debug, Serialize, Deserialize)]
pub struct IKInfo<'a> {
    pub identifier: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    pub description: &'a str,
    pub personalities: HashMap<&'a str, HashMap<String, dt::OSValue>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IKCache<'a> {
    #[serde(borrow)]
    pub infos: Vec<IKInfo<'a>>,
    pub payloads: HashMap<&'a str, &'a [u8]>,
}
