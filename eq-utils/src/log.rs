pub use crate::eq_log;

#[cfg(feature = "std")]
#[macro_export]
macro_rules! eq_log {
  ($($arg:tt)+) => {
    println!($($arg)+);
  }
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! eq_log {
  ($($arg:tt)+) => {
        debug::warn!($($arg)+)
    }
}
