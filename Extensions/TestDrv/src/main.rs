// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
// #[macro_use]
extern crate alloc;

use alloc::string::String;
use core::fmt::Write;

use iridium_kit::{port::Port, syscall::SystemCall};
use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};

mod allocator;
mod logger;
mod panic;

#[used]
#[no_mangle]
static __stack_chk_guard: u64 = 0x595E_9FBD_94FD_A766;

#[no_mangle]
extern "C" fn __stack_chk_fail() {
    panic!("stack check failure");
}

// const PS2_SERVICE: uuid::Uuid = uuid!("e8f08fbc-b0a3-4365-b91e-12dbfeec6586");

#[derive(IntoPrimitive)]
#[repr(u8)]
enum PS2CtlCmd {
    ReadControllerCfg = 0x20,
    WriteControllerCfg = 0x60,
}

#[bitfield(bits = 8)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
struct Ps2Cfg {
    pub port1_intr: bool,
    pub port2_intr: bool,
    pub post_pass: bool,
    #[skip]
    __: bool,
    pub port1_clock: bool,
    pub port2_clock: bool,
    pub port1_translation: bool,
    #[skip]
    __: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
enum Ps2Event {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    BackSpace,
    Pressed(char),
    Released(char),
    Other(u8),
}

struct PS2Ctl {
    data_port: Port<u8, u8>,
    sts_or_cmd_reg: Port<u8, u8>,
}

impl PS2Ctl {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data_port: Port::new(0x60),
            sts_or_cmd_reg: Port::new(0x64),
            // queue: VecDeque::new(),
        }
    }

    #[inline]
    fn output_full(&self) -> bool {
        unsafe { (self.sts_or_cmd_reg.read() & 1) != 0 }
    }

    #[inline]
    fn input_full(&self) -> bool {
        unsafe { (self.sts_or_cmd_reg.read() & (1 << 1)) != 0 }
    }

    #[inline]
    fn send_cmd(&self, cmd: PS2CtlCmd, wait_for_ack: bool) {
        unsafe {
            self.sts_or_cmd_reg.write(cmd.into());
            if wait_for_ack {
                while self.data_port.read() != 0xFA {}
            }
        }
    }

    pub fn init(&self) {
        while self.output_full() {
            let _ = unsafe { self.data_port.read() };
        }

        self.send_cmd(PS2CtlCmd::ReadControllerCfg, false);
        while !self.output_full() {}

        let cfg = unsafe {
            Ps2Cfg::from(self.data_port.read())
                .with_port1_intr(true)
                .with_port2_intr(false)
                .with_port1_translation(true)
        };
        unsafe { SystemCall::register_irq_handler(1).unwrap() }
        self.send_cmd(PS2CtlCmd::WriteControllerCfg, false);
        unsafe { self.data_port.write(cfg.into()) }
        while self.input_full() {}
    }
}

#[no_mangle]
extern "C" fn _start() -> ! {
    logger::init();
    // unsafe { SystemCall::register_provider(PS2_SERVICE).unwrap() };
    // let target = unsafe { SystemCall::get_providing_process(PS2_SERVICE).unwrap() };

    let this = PS2Ctl::new();
    this.init();
    let mut s = String::new();
    write!(logger::KWriter, "> ").unwrap();
    loop {
        let Some(msg) = (unsafe { SystemCall::receive_message().unwrap() }) else {
            unsafe { SystemCall::skip() };
            continue;
        };
        if msg.proc_id == 0 {
            while this.output_full() {
                let event = match unsafe { this.data_port.read() } {
                    0xE => Ps2Event::BackSpace,
                    v @ 0x2..=0xB => {
                        Ps2Event::Pressed("1234567890".chars().nth(v as usize - 0x2).unwrap())
                    }
                    0x1C => Ps2Event::Pressed('\n'),
                    v @ 0x10..=0x1C => {
                        Ps2Event::Pressed("qwertyuiop".chars().nth(v as usize - 0x10).unwrap())
                    }
                    v @ 0x1E..=0x26 => {
                        Ps2Event::Pressed("asdfghjkl".chars().nth(v as usize - 0x1E).unwrap())
                    }
                    0x29 => Ps2Event::Pressed('0'),
                    v @ 0x2C..=0x32 => {
                        Ps2Event::Pressed("zxcvbnm".chars().nth(v as usize - 0x2C).unwrap())
                    }
                    0x39 => Ps2Event::Pressed(' '),
                    v => Ps2Event::Other(v),
                };

                if let Ps2Event::Pressed(ch) = event {
                    write!(logger::KWriter, "{ch}").unwrap();
                    if ch != '\n' {
                        s.push(ch);
                    } else {
                        info!("You typed: {}", s);
                        write!(logger::KWriter, "> ").unwrap();
                        s.clear();
                    }
                };
            }
        }
        unsafe { SystemCall::ack_message(msg.id).unwrap() }
    }
}