use ctru_sys::{svcGetProcessInfo, Handle};

use crate::error::{Result, ResultCode};
#[repr(u32)]
pub enum ProcessInfoType {
    TotalPhysicalMemory = 0,
    _Mem1,
    _Mem2,
    _Mem3,
    NumHandles,
    HighestSimultaneousHandles,
    _KProcess0x234,
    NumThreads,
    MaxThreads,
    // Syscalls 9-18 are stubbed out [https://www.3dbrew.org/wiki/8.0.0-18]
    MemRegion = 19,
    LinearMemoryOffset,
    // New3DS Only
    QTMOffset,
    QTMBase,
    QTMSize,
}

// TODO: Where should this live?
pub const fn get_current_process() -> Handle {
    ctru_sys::CUR_PROCESS_HANDLE
}

pub fn get_process_info(process: Handle, info_type: ProcessInfoType) -> Result<i64> {
    let mut output: i64 = 0;
    ResultCode(unsafe { svcGetProcessInfo(&mut output, process, info_type as u32) })?;
    Ok(output)
}
