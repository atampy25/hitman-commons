/// Create a const [`RuntimeID`](crate::metadata::RuntimeID) from a resource path.
///
/// Same as the [`RuntimeID::from_path`](crate::metadata::RuntimeID::from_path) function, but is evaluated at compile time so that it can be used in const contexts. 
///
/// # Panics (const-eval)
/// Panics if the given path is longer than 512 bytes.
///
/// # Example
/// ```rust
/// use hitman_commons::rid;
/// use hitman_commons::metadata::RuntimeID;
///
/// const ID: RuntimeID = rid!("[your:/resource/path].xx_here");
/// ```

#[macro_export]
macro_rules! rid {
    ($path:expr) => {{
        const __S: &str = $path;
        const __ID: u64 = {
            let bytes = __S.as_bytes();

            let mut lower = [0u8; 512];
            let mut i: usize = 0;
            if bytes.len() > lower.len() {
                panic!("rid!: path too long for const buffer (max 512 bytes)");
            }

            while i < bytes.len() {
                let b = bytes[i];
                lower[i] = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
                i += 1;
            }

            let (prefix, _) = lower.split_at(bytes.len());
            let digest: [u8; 16] = lhash::md5(prefix);

            let mut val: u64 = 0;
            let mut j: usize = 1;
            while j < 8 {
                val |= (digest[j] as u64) << (8 * (7 - j));
                j += 1;
            }
            val
        };

        $crate::metadata::RuntimeID::from_u64_const(__ID)
    }};
}

/// Create a const [`RuntimeID`](crate::metadata::RuntimeID) from a numeric runtime resource id.
///
/// Constructs a `RuntimeID` from it's hash, evaluated at compile time so it can be used in const contexts.
///
/// # Panics (const-eval)
/// Panics if the value is not a valid `RuntimeID` (too large).
///
/// # Example
/// ```rust
/// use hitman_commons::rrid;
/// use hitman_commons::metadata::RuntimeID;
///
/// const ID: RuntimeID = rrid!(0x00123456789ABCDE);
/// ```
#[macro_export]
macro_rules! rrid {
    ($val:expr) => {{
        const __VAL: u64 = $val;
        $crate::metadata::RuntimeID::from_u64_const(__VAL)
    }};
}