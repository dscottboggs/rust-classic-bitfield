mod context;

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemEnum;

use context::{BitfieldEnumCtx, ReprType};

fn impl_bitfield_enum(ctx: BitfieldEnumCtx) -> TokenStream {
    let type_name = &ctx.ident;
    let repr_type = &*ctx.repr_type;
    let visibility = &ctx.vis;
    let struct_def = quote! {
        struct #type_name(#repr_type);
    };
    let constant_values = ctx.constant_values();
    let has_methods = ctx.has_methods();
    let impl_from_and_deref = ctx.impl_from_and_deref();
    let impl_bitand = ctx.impl_bitand();
    let impl_bitor = ctx.impl_bitor();
    let impl_partial_eq_ord = ctx.impl_partial_eq_ord();
    let impl_debug = ctx.impl_debug();

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #visibility #struct_def

        impl #type_name {
            #(#constant_values)*

            #(#has_methods)*
        }

        #impl_from_and_deref

        #impl_bitand

        #impl_bitor

        #impl_partial_eq_ord

        #impl_debug
    }
    .into()
}

#[proc_macro_attribute]
pub fn bitfield_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    let repr_type: ReprType = syn::parse(attr).expect("attribute should be empty or (as T)");
    let enum_def: ItemEnum = syn::parse(item).expect("failed to parse input");
    let ctx = BitfieldEnumCtx {
        repr_type,
        enum_def,
    };
    impl_bitfield_enum(ctx)
}
