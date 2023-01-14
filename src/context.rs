use std::ops::Deref;

use derive_deref::Deref;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parse, spanned::Spanned, Ident, ItemEnum, Token, Type};

#[derive(Deref)]
pub(crate) struct ReprType(Type);

impl ReprType {
    #[allow(dead_code)]
    /// TODO allow custom repr types
    pub(crate) fn try_convert(&self, value: usize) -> impl ToTokens {
        if let Type::Path(ref path) = self.0 {
            match path.to_token_stream().to_string().as_str() {
                "i8" => quote! { i8::try_from(#value) },
                "i16" => quote! { i16::try_from(#value) },
                "i32" => quote! { i32::try_from(#value) },
                "i64" => quote! { i64::try_from(#value) },
                "i128" => quote! { i128::try_from(#value) },
                "isize" => quote! { isize::try_from(#value) },
                "u8" => quote! { u8::try_from(#value) },
                "u16" => quote! { u16::try_from(#value) },
                "u32" => quote! { u32::try_from(#value) },
                "u64" => quote! { u64::try_from(#value) },
                "u128" => quote! { u128::try_from(#value) },
                "usize" => quote! { usize::try_from(#value) },
                unknown => panic!("type {unknown} is either inappropriate or not yet implemented"),
            }
        } else {
            panic!(
                "type was not of type path, it was {}",
                self.0.to_token_stream()
            )
        }
    }
}

impl Parse for ReprType {
    #[allow(unreachable_code)] // TODO remove this allow
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(Self(Type::Verbatim(quote! { u64 })))
        } else {
            todo!("representing as types other than u64 is not yet implemented. Help wanted!");
            input.parse::<Token![as]>()?;
            Ok(Self(input.parse()?))
        }
    }
}

pub(crate) struct BitfieldEnumCtx {
    pub(crate) enum_def: ItemEnum,
    pub(crate) repr_type: ReprType,
}

impl Deref for BitfieldEnumCtx {
    type Target = ItemEnum;
    fn deref(&self) -> &Self::Target {
        &self.enum_def
    }
}

impl BitfieldEnumCtx {
    pub(crate) fn constant_values(&self) -> Vec<impl ToTokens> {
        self.variants
            .iter()
            .enumerate()
            .map(|(i, variant)| {
                let name = &variant.ident;
                let name = Ident::new(&name.to_string().to_uppercase(), name.span());
                let i = 1u64 << i;
                // TODO convert `i` to whatever type is specified in the `#[repr(as T)]` annotation
                let repr = variant
                    .attrs
                    .iter()
                    .find_map(|attr| {
                        attr.path.get_ident().and_then(|ident| {
                            if ident == "repr" {
                                attr.parse_args().ok()
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or_else(|| quote! { #i });
                let type_name = &self.ident;
                quote! {
                    const #name: #type_name = #type_name(#repr);
                }
            })
            .collect()
    }

    /// A method like `has_x` for each variant like `X`.
    pub(crate) fn has_methods(&self) -> Vec<impl ToTokens> {
        self.variants
            .iter()
            .map(|variant| {
                let variant_name = &variant.ident;
                let variant_name_str = variant_name.to_string();
                let fn_name = format_ident!("has_{}", variant_name_str.to_lowercase());
                let variant_name = Ident::new(
                    variant_name.to_string().to_uppercase().as_str(),
                    variant.span(),
                );
                quote! {
                    /// Check if this bitfield has the #variant_name flag set.
                    fn #fn_name(self) -> bool {
                        (self & Self::#variant_name) == Self::#variant_name
                    }
                }
            })
            .collect()
    }

    /// Impls for From<repr_type>/Into<type_name> and vice-versa, Deref and
    /// DerefMut of the internal value.
    pub(crate) fn impl_from_and_deref(&self) -> impl ToTokens {
        let type_name = &self.ident;
        let repr_type = &*self.repr_type;
        quote! {
            impl From<#repr_type> for #type_name {
                fn from(value: #repr_type) -> Self {
                    Self(value)
                }
            }

            impl From<#type_name> for #repr_type {
                fn from(value: #type_name) -> Self {
                    value.0
                }
            }

            impl std::ops::Deref for #type_name {
                type Target = #repr_type;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl std::ops::DerefMut for #type_name {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        }
    }

    /// Impls for bitwise-and operations
    pub(crate) fn impl_bitand(&self) -> impl ToTokens {
        let type_name = &self.ident;
        let repr_type = &*self.repr_type;
        quote! {
            impl std::ops::BitAnd<#type_name> for #type_name {
                type Output = #type_name;
                fn bitand(self, rhs: #type_name) -> Self::Output {
                    Self(self.0 & rhs.0)
                }
            }

            impl std::ops::BitAnd<#repr_type> for #type_name {
                type Output = #type_name;
                fn bitand(self, rhs: #repr_type) -> Self::Output {
                    Self(self.0 & rhs)
                }
            }

            impl std::ops::BitAnd<#type_name> for #repr_type {
                type Output = #type_name;
                fn bitand(self, rhs: #type_name) -> Self::Output {
                    #type_name(self & rhs.0)
                }
            }
        }
    }

    /// Impls for bitwise-or operations
    pub(crate) fn impl_bitor(&self) -> impl ToTokens {
        let type_name = &self.ident;
        let repr_type = &*self.repr_type;
        quote! {
            impl std::ops::BitOr<#type_name> for #repr_type {
                type Output = #type_name;
                fn bitor(self, rhs: #type_name) -> Self::Output {
                    #type_name(self | rhs.0)
                }
            }

            impl std::ops::BitOr<#repr_type> for #type_name {
                type Output = #type_name;
                fn bitor(self, rhs: #repr_type) -> Self::Output {
                    Self(self.0 | rhs)
                }
            }

            impl std::ops::BitOr<#type_name> for #type_name {
                type Output = #type_name;
                fn bitor(self, rhs: #type_name) -> Self::Output {
                    Self(self.0 | rhs.0)
                }
            }
        }
    }

    pub(crate) fn impl_not(&self) -> impl ToTokens {
        let type_name = &self.ident;
        quote! {
            impl Not for #type_name {
                type Output = Self;

                fn not(&self) -> Self::Output {
                    Self(!self.0)
                }
            }
        }
    }

    pub(crate) fn impl_partial_eq_ord(&self) -> impl ToTokens {
        let type_name = &self.ident;
        let repr_type = &*self.repr_type;

        quote! {
            impl core::cmp::PartialEq<#repr_type> for #type_name {
                fn eq(&self, other: &#repr_type) -> bool {
                    self.0 == *other
                }
            }

            impl core::cmp::PartialOrd<#repr_type> for #type_name {
                fn partial_cmp(&self, other: &#repr_type) -> Option<std::cmp::Ordering> {
                    self.0.partial_cmp(other)
                }
            }
        }
    }

    pub(crate) fn impl_debug(&self) -> impl ToTokens {
        let type_name = &self.ident;
        let check_each_variant = self.variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            let has_method = format_ident!("has_{}", variant_name.to_string().to_lowercase());
            quote! {
                if self.#has_method() {
                    if at_least_one {
                        write!(f, " | ")?;
                    }
                    at_least_one = true;
                    write!(f, concat!(stringify!(#type_name), "::", stringify!(#variant_name)))?;
                }
            }
        });
        quote! {
            impl std::fmt::Debug for #type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let mut at_least_one = false;

                    #(#check_each_variant)*

                    if !at_least_one {
                        write!(f, concat!(stringify!(#type_name), "({})"), self.0)?;
                    }
                    Ok(())
                }
            }
        }
    }
}
