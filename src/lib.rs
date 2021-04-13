//! Zero-terminated C string literals.
#![doc(html_root_url = "https://docs.rs/neodyn_db/0.1.0")]

use proc_macro::TokenStream;
use syn::{ parse_macro_input, Lit, LitByteStr };
use quote::quote;

/// Given a Rust string or byte string literal, this macro
/// generates an expression of type `&'static CStr` that is
/// properly 0-terminated and ensured not to contain any
/// internal NUL (0) bytes. The conversion is zero-cost and
/// it may even become `const` in the future, provided that
/// `CStr::from_bytes_with_nul_unchecked()` becomes `const`.
///
/// ### Examples:
///
/// ```
/// use zstr::zstr;
///
/// // Works with Unicode characters and escapes in Rust strings
/// let c_str_1 = zstr!("Hello 🎉");
/// assert_eq!(c_str_1.to_bytes(), b"Hello \xf0\x9f\x8e\x89");
/// assert_eq!(c_str_1.to_bytes_with_nul(), b"Hello \xf0\x9f\x8e\x89\x00");
///
/// let c_str_2 = zstr!("Hello \u{1F389}");
/// assert_eq!(c_str_1, c_str_2);
///
/// let c_str_3 = zstr!(b"hello\x20ASCII");
/// assert_eq!(c_str_3.to_bytes(), b"hello ASCII");
/// assert_eq!(c_str_3.to_bytes_with_nul(), b"hello ASCII\x00");
/// ```
///
/// Strings with embedded NUL (zero) bytes are not allowed:
///
/// ```compile_fail
/// # use zstr::zstr;
/// #
/// let invalid_1 = zstr!("null here: \x00 is forbidden");
/// let invalid_2 = zstr!("also at the end: \0");
/// let invalid_3 = zstr!(b"and in byte \x00 strings too");
/// ```
#[proc_macro]
pub fn zstr(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as Lit);
    let span = literal.span();

    let mut bytes = match literal {
        Lit::Str(lit) => lit.value().into_bytes(),
        Lit::ByteStr(lit) => lit.value(),
        _ => panic!("expected a string or byte string literal"),
    };

    // Ensure that no 0 byte is in the string literal, as that
    // would cause inconsistencies in the length of the string.
    if let Some(index) = bytes.iter().position(|&b| b == 0x00) {
        panic!("C string contains an embedded NUL byte at index {}", index);
    }

    // Add the terminating 0.
    bytes.reserve_exact(1);
    bytes.push(0x00);

    // Convert to a byte string literal.
    let bstr = LitByteStr::new(&bytes, span);

    // Expand to an expression of type `&'static CStr`.
    let expanded = quote!{
        // SAFETY: the input is NUL-terminated and it is ensured
        // that it does not contain any other, internal NUL bytes.
        unsafe {
            ::std::ffi::CStr::from_bytes_with_nul_unchecked(#bstr)
        }
    };

    TokenStream::from(expanded)
}
