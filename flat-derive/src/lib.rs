use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// This derive macro can be unsafe. FlatBuffer can only be derived for "flat"
/// types, i.e. those that contain no references and can be represented in a
/// single buffer.
#[proc_macro_derive(FlatBuffer)]
pub fn derive_flat_buffer(toks: TokenStream) -> TokenStream {
    let input = parse_macro_input!(toks as DeriveInput);
    let name = input.ident;

    let out = quote! {
        unsafe impl ::flat::FlatBuffer for #name {
            #[inline]
            fn size(&self) -> usize {
                std::mem::size_of::<Self>()
            }

            #[inline]
            fn ptr(&self) -> *const u8 {
                (self as *const Self) as *const _
            }

            #[inline]
            fn ptr_mut(&mut self) -> *mut u8 {
                (self as *mut Self) as *mut _
            }

            #[inline]
            fn type_id() -> u64 {
                use ::std::hash::{Hash, Hasher};
                let id = ::std::any::TypeId::of::<Self>();
                let mut hasher = ::std::collections::hash_map::DefaultHasher::new();
                id.hash(&mut hasher);
                hasher.finish()
            }
        }
    };

    TokenStream::from(out)
}
