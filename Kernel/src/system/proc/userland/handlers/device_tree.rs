// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::ops::ControlFlow;

use tungstenkit::{syscall::OSDTEntryReqType, TerminationReason};

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn get_info(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let sys_state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    let dt_index = sys_state.dt_index.as_ref().unwrap().read();
    let Ok(info_type) = OSDTEntryReqType::try_from(state.rdx) else {
        return ControlFlow::Break(Some(TerminationReason::MalformedArgument));
    };
    let Some(ent) = dt_index.get(&state.rsi) else {
        return ControlFlow::Break(Some(TerminationReason::NotFound));
    };
    let data = match info_type {
        OSDTEntryReqType::Parent => postcard::to_allocvec(&ent.lock().parent),
        OSDTEntryReqType::Children => postcard::to_allocvec(&ent.lock().children),
        OSDTEntryReqType::Properties => postcard::to_allocvec(&ent.lock().properties),
        OSDTEntryReqType::Property => {
            let Ok(k) = core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(state.rcx as *const _, state.r8 as _)
            }) else {
                return ControlFlow::Break(Some(TerminationReason::MalformedAddress));
            };
            postcard::to_allocvec(&ent.lock().properties.get(k))
        }
    }
    .unwrap()
    .leak();

    state.rax = scheduler
        .current_process_mut()
        .unwrap()
        .track_kernelside_alloc(data.as_ptr() as _, data.len() as _);
    state.rdi = data.len() as _;

    ControlFlow::Continue(())
}

pub fn new_entry(state: &mut RegisterState) -> ControlFlow<Option<TerminationReason>> {
    let sys_state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    let dt_index = sys_state.dt_index.as_ref().unwrap();
    let new = {
        let dt_index = dt_index.read();
        let Some(parent) = dt_index.get(&state.rsi) else {
            return ControlFlow::Break(Some(TerminationReason::NotFound));
        };
        let v = crate::system::state::OSDTEntry {
            id: sys_state.dt_id_gen.as_ref().unwrap().lock().next(),
            parent: Some(state.rsi.into()),
            ..Default::default()
        };
        parent.lock().children.push(v.id.into());
        v
    };
    state.rax = new.id;
    dt_index.write().insert(new.id, new.into());

    ControlFlow::Continue(())
}
