//! Zero-terminated C string literals.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{ parse2, Error, Lit, LitByteStr };
use quote::quote_spanned;

/// Given a Rust string or byte string literal, this macro
/// generates an expression of type `&'static CStr` that is
/// properly 0-terminated and ensured not to contain any
/// internal NUL (0) bytes. The conversion is zero-cost, and
/// the resulting expression can be used in `const` context.
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
/// let invalid_4 = zstr!(b"at the end of byte strings: \0");
/// ```
#[proc_macro]
pub fn zstr(input: TokenStream) -> TokenStream {
    expand_zstr(input.into())
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Performs the actual expansion of `zstr!()`.
fn expand_zstr(input: TokenStream2) -> Result<TokenStream2, Error> {
    let literal: Lit = parse2(input)?;
    let span = literal.span();

    let mut bytes = match literal {
        Lit::Str(lit) => lit.value().into_bytes(),
        Lit::ByteStr(lit) => lit.value(),
        _ => return Err(Error::new(span, "expected a string or byte string literal")),
    };

    // Ensure that no 0 byte is in the string literal, as that
    // would cause inconsistencies in the length of the string.
    if let Some(index) = bytes.iter().position(|&b| b == 0x00) {
        let message = format!("C string contains an embedded NUL byte at index {}", index);
        return Err(Error::new(span, message));
    }

    // Add the terminating 0.
    bytes.reserve_exact(1);
    bytes.push(0x00);

    // Convert to a byte string literal.
    let bstr = LitByteStr::new(&bytes, span);

    // Expand to an expression of type `&'static CStr`.
    Ok(quote_spanned!{
        // SAFETY: the input is NUL-terminated and it is ensured
        // that it does not contain any other, internal NUL bytes.
        span => unsafe {
            ::std::ffi::CStr::from_bytes_with_nul_unchecked(#bstr)
        }
    })
}
