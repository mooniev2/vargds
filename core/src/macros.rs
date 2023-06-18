macro_rules! mb {
    ($val:expr) => {
        (($val) << 20)
    };
}

macro_rules! kb {
    ($val:expr) => {
        (($val) << 10)
    };
}

macro_rules! get_bit {
    ($val:expr, $bit:literal) => {
        (($val >> $bit) & 0b1 != 0)
    };
}

macro_rules! set_bit {
    ($val:expr, $bit:literal) => {{
        $val = ($val & !b!($bit)) | ((1) << $bit)
    }};
}

macro_rules! unset_bit {
    ($val:expr, $bit:literal) => {{
        $val = ($val & !b!($bit))
    }};
}

macro_rules! toggle_bit {
    ($val:expr, $bit:literal, $bool:expr) => {{
        $val = ($val & !(1 << $bit)) | ((if $bool { 1 } else { 0 }) << $bit)
    }};
}

macro_rules! b {
    ($bit:literal) => {
        (1 << ($bit))
    };
}

// logging macros

#[cfg(feature = "log")]
macro_rules! log {
    ($($arg:tt)+) => { ::slog::log!($($arg)+) };
}
#[cfg(not(feature = "log"))]
macro_rules! log {
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log")]
macro_rules! info {
    ($($arg:tt)+) => { ::slog::info!($($arg)+) };
}
#[cfg(not(feature = "log"))]
macro_rules! info {
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log")]
macro_rules! warn {
    ($($arg:tt)+) => { ::slog::warn!($($arg)+) };
}
#[cfg(not(feature = "log"))]
macro_rules! warn {
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log")]
macro_rules! error {
    ($($arg:tt)+) => { ::slog::error!($($arg)+) };
}
#[cfg(not(feature = "log"))]
macro_rules! error {
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log")]
macro_rules! debug {
    ($($arg:tt)+) => { ::slog::debug!($($arg)+) };
}
#[cfg(not(feature = "log"))]
macro_rules! debug {
    ($($arg:tt)+) => {};
}

#[cfg(feature = "log")]
macro_rules! trace {
    ($($arg:tt)+) => { ::slog::trace!($($arg)+) };
}
#[cfg(not(feature = "log"))]
macro_rules! trace {
    ($($arg:tt)+) => {};
}
