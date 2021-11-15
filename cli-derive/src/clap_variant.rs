use crate::types::{StructArgs, FieldArgs};
use proc_macro2::TokenStream;
use quote::{quote};
use syn::{Ident, Type};


pub fn gen(args: &StructArgs) -> TokenStream {
    let struct_ident = &args.ident;
    let name = format!("{}ClapVariant", struct_ident);
    let name = syn::Ident::new(&name, struct_ident.span());
    let (passthru, fields) = gen_clap_internals(args);

    quote! {
        // #[derive(Parser)]
        struct #name {
            #(#fields)*
        }

        #passthru

        impl near_cli_visual::types::ClapVariant for #struct_ident {
            type Clap = #name;
        }
    }
}

fn gen_clap_internals(args : &StructArgs) -> (TokenStream, Vec<TokenStream>) {
    let StructArgs {
        ident: struct_ident,
        generics: _,
        data: _,
    } = args;

    // let mut passthru = None;
    let mut sub_args = SubcommandArgs {
        ident: struct_ident.clone(),

        // by default, if no one specifies `single`, then there's no passthru
        // code to generate for this clap variant.
        passthru: quote!(),
    };

    let fields = args.fields().into_iter().map(|f| {
        let FieldArgs {
            ident,
            ty,
            single,
            subcommand,
            ..
        } = f;

        // TODO: potential do not generate clap variant option if we skip it.
        // let field_ty = if f.skip {
        //     quote!(#field_ty)
        // } else {
        //     quote! {
        //         Option<#field_ty>
        //     }
        // };

        let mut ty = quote!(#ty);
        let mut qualifiers = quote! {};
        if *subcommand {
            // qualifiers = quote! { #[clap(subcommand)] };
            qualifiers = quote! {};
            if *single {
                let (ident, code) = gen_clap_enum_pass(struct_ident, &ty);
                ty = quote!(#ident);
                sub_args.ident = ident;
                sub_args.passthru = code;
            }
        }

        let ty = quote! { Option<#ty> };
        let field = if let Some(ident) = ident {
            quote! { #ident: #ty, }
        } else {
            ty
        };

        quote! {
            #qualifiers
            #field
        }
    })
    .collect();

    (sub_args.passthru, fields)
}

struct SubcommandArgs {
    ident: Ident,
    passthru: TokenStream,
}

fn gen_clap_enum_pass(struct_ident: &Ident, ty: &TokenStream) -> (Ident, TokenStream) {
    let passthru_ident = format!("{}ClapVariantPassThru", struct_ident);
    let passthru_ident = syn::Ident::new(&passthru_ident, struct_ident.span());
    let code = quote! {
        // #[derive(Parser)]
        enum #passthru_ident {
            PassThru(#ty)
        }
    };

    (passthru_ident, code)
}
