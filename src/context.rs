use core::ops::Deref;

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
                    .unwrap_or(quote! { #i });
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
                    #[doc=concat!("Check if this bitfield has the ", stringify!(#variant_name), " flag set.")]
                    fn #fn_name(self) -> bool {
                        (self & Self::#variant_name) == Self::#variant_name
                    }
                }
            })
            .collect()
    }

    /// Adds `.with()` and `.without()` methods
    pub(crate) fn with_and_without(&self) -> impl ToTokens {
        quote! {
            /// Combines this flag with `other`.
            fn with(self, other: impl Into<Self>) -> Self
            {
                self | other.into()
            }

            /// Returns this value with `other` ensured to be unset.
            fn without(self, other: impl Into<Self>) -> Self
            {
                self & (!(other.into()))
            }
        }
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

            impl core::ops::Deref for #type_name {
                type Target = #repr_type;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl core::ops::DerefMut for #type_name {
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
            impl core::ops::BitAnd<#type_name> for #type_name {
                type Output = #type_name;
                fn bitand(self, rhs: #type_name) -> Self::Output {
                    Self(self.0 & rhs.0)
                }
            }

            impl core::ops::BitAnd<#repr_type> for #type_name {
                type Output = #type_name;
                fn bitand(self, rhs: #repr_type) -> Self::Output {
                    Self(self.0 & rhs)
                }
            }

            impl core::ops::BitAnd<#type_name> for #repr_type {
                type Output = #type_name;
                fn bitand(self, rhs: #type_name) -> Self::Output {
                    #type_name(self & rhs.0)
                }
            }

            impl core::ops::BitAndAssign for #type_name {
                fn bitand_assign(&mut self, rhs: Self) {
                    self.0 &= rhs.0
                }
            }

            impl core::ops::BitAndAssign<#repr_type> for #type_name {
                fn bitand_assign(&mut self, rhs: #repr_type) {
                    self.0 &= rhs;
                }
            }
        }
    }

    /// Impls for bitwise-or operations
    pub(crate) fn impl_bitor(&self) -> impl ToTokens {
        let type_name = &self.ident;
        let repr_type = &*self.repr_type;
        quote! {
            impl core::ops::BitOr<#type_name> for #repr_type {
                type Output = #type_name;
                fn bitor(self, rhs: #type_name) -> Self::Output {
                    #type_name(self | rhs.0)
                }
            }

            impl core::ops::BitOr<#repr_type> for #type_name {
                type Output = #type_name;
                fn bitor(self, rhs: #repr_type) -> Self::Output {
                    Self(self.0 | rhs)
                }
            }

            impl core::ops::BitOr<#type_name> for #type_name {
                type Output = #type_name;
                fn bitor(self, rhs: #type_name) -> Self::Output {
                    Self(self.0 | rhs.0)
                }
            }

            impl core::ops::BitOrAssign for #type_name {
                fn bitor_assign(&mut self, rhs: Self) {
                    self.0 |= rhs.0
                }
            }

            impl core::ops::BitOrAssign<#repr_type> for #type_name {
                fn bitor_assign(&mut self, rhs: #repr_type) {
                    self.0 |= rhs;
                }
            }
        }
    }

    pub(crate) fn impl_not(&self) -> impl ToTokens {
        let type_name = &self.ident;
        quote! {
            impl core::ops::Not for #type_name {
                type Output = Self;

                fn not(self) -> Self::Output {
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

    fn name_value_pairs(&self) -> (Vec<String>, Vec<impl ToTokens>) {
        self.variants
            .iter()
            .map(|variant| {
                let name = &variant.ident;
                (name.to_string().to_uppercase(), quote! { #name })
            })
            .unzip()
    }

    pub(crate) fn impl_iter_variants(&self) -> impl ToTokens {
        let (key, value) = self.name_value_pairs();
        let type_name = &self.ident;
        let vis = &self.vis;
        quote! {
            /// The name of each variant
            #vis const fn variant_names() -> &'static [&'static str] {
                &[
                    #(#key),*
                ]
            }

            /// Each value defined by a name
            #vis const fn variant_values() -> &'static [Self] {
                &[
                    #(Self::#value),*
                ]
            }

            /// The name of each variant along with the corresponding value.
            #vis const fn variant_pairs() -> &'static [(&'static str, Self)] {
                &[
                    #(
                        (#key, Self::#value)
                    ),*
                ]
            }

            #[doc=concat!("An instance of `", stringify!(#type_name), "` with all named variants set on")]
            #vis fn all_set() -> Self {
                Self(0) | #( Self::#value )|*
            }

            #[doc=concat!("The names of each variant which is set on this instance of `", stringify!(#type_name), "`")]
            #vis fn names_of_set_variants(self) -> Vec<&'static str> {
                let mut names = vec![];
                #(
                    if (self & Self::#value) == Self::#value {
                        names.push(#key);
                    }
                )*
                names
            }
        }
    }

    #[cfg(feature = "serde-as-number")]
    pub(crate) fn impl_serde(&self) -> impl ToTokens {
        let type_name = &self.ident;
        let serialize_method = Ident::new(
            &format!("serialize_{}", &self.repr_type.to_token_stream()),
            syn::__private::Span::call_site(),
        );
        let deserialize_method = Ident::new(
            &format!("deserialize_{}", self.repr_type.to_token_stream()),
            syn::__private::Span::call_site(),
        );
        let visit_method = Ident::new(
            &format!("visit_{}", self.repr_type.to_token_stream()),
            syn::__private::Span::call_site(),
        );
        quote! {
            impl serde::Serialize for #type_name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.#serialize_method(self.0)
                }
            }

            impl<'de> serde::Deserialize<'de> for #type_name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct MyVisitor;

                    impl<'v> serde::de::Visitor<'v> for MyVisitor {
                        type Value = #type_name;
                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(formatter, "integer")
                        }

                        fn #visit_method<E>(self, v: u64) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            Ok(#type_name(v))
                        }
                    }

                    deserializer.#deserialize_method(MyVisitor)
                }
            }
        }
    }

    #[cfg(feature = "serde-as-names")]
    pub(crate) fn impl_serde(&self) -> impl ToTokens {
        let type_name = &self.ident;
        let (variant, has_method): &(Vec<_>, Vec<_>) = &self
            .variants
            .iter()
            .map(|vairant| {
                let name = vairant.ident.to_string();
                let has_method = Ident::new(&format!("has_{}", name.to_lowercase()), name.span());
                (name, has_method)
            })
            .unzip();
        let (key, value) = self.name_value_pairs();
        quote! {
            impl serde::Serialize for #type_name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeSeq;

                    let mut seq = serializer.serialize_seq(None)?;
                    #(
                        if self.#has_method() {
                            seq.serialize_element(#variant);
                        }
                    )*
                    seq.end()
                }
            }

            impl<'de> serde::Deserialize<'de> for #type_name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct MyVisitor;

                    impl<'v> serde::de::Visitor<'v> for MyVisitor {
                        type Value = #type_name;
                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(formatter, "a list of any of these values: {:?}", #type_name::variant_names())
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'v>,
                        {
                            let mut value = #type_name(0);

                            while let Some(member) = seq.next_element()? {
                                match member {
                                    #(#key => value |= #type_name::#value),*,
                                    unrecognized => {
                                        return Err(de::Error::unknown_variant(
                                            unrecognized,
                                            #type_name::variant_names(),
                                        ));
                                    }
                                }
                            }

                            Ok(value)
                        }
                    }

                    deserializer.deserialize_seq(MyVisitor)
                }
            }
        }
    }
    #[cfg(not(any(feature = "serde-as-number", feature = "serde-as-names")))]
    pub(crate) fn impl_serde(&self) -> impl ToTokens {
        quote! {}
    }
}
