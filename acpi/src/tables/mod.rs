// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::any::type_name;

pub mod bgrt;
pub mod hpet;
pub mod madt;
pub mod mcfg;
pub mod rsdp;
pub mod rsdt;
pub mod xsdt;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SDTHeader {
    signature: [u8; 4],
    length: u32,
    pub revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    pub oem_revision: u32,
    creator_id: [u8; 4],
    pub creator_revision: u32,
}

impl SDTHeader {
    #[must_use]
    pub fn validate(&self) -> bool {
        let bytes = unsafe {
            core::slice::from_raw_parts((self as *const Self).cast::<u8>(), self.length())
        };
        let sum = bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte));

        sum == 0
    }

    #[must_use]
    pub fn signature(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.signature).trim() }
    }

    #[must_use]
    pub const fn length(&self) -> usize {
        self.length as usize
    }

    #[must_use]
    pub fn oem_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_id).trim() }
    }

    #[must_use]
    pub fn oem_table_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_table_id).trim() }
    }

    #[must_use]
    pub fn creator_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.creator_id).trim() }
    }
}

impl core::fmt::Debug for SDTHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let rev = self.oem_revision;
        let cr_rev = self.creator_revision;
        f.debug_struct(type_name::<Self>())
            .field("valid", &self.validate())
            .field("signature", &self.signature())
            .field("length", &self.length())
            .field("revision", &self.revision)
            .field("oem_id", &self.oem_id())
            .field("oem_table_id", &self.oem_table_id())
            .field("oem_revision", &rev)
            .field("creator_id", &self.creator_id())
            .field("creator_revision", &cr_rev)
            .finish()
    }
}
