#[macro_export]
macro_rules! log {
    ($($tokens: tt)*) => {
        for line in format!($($tokens)*).lines() {
            println!("cargo::warning={}", line)
        }
    }
}
