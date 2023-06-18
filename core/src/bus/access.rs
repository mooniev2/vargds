pub trait Access {
    const CPU: bool;
    const DEBUG: bool;
}

pub struct CPUAccess;

impl Access for CPUAccess {
    const CPU: bool = true;
    const DEBUG: bool = false;
}

pub struct DebugAccess;

impl Access for DebugAccess {
    const CPU: bool = false;
    const DEBUG: bool = true;
}
