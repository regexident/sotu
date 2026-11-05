use std::collections::HashSet;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{
    Data, DeriveInput, Fields, GenericParam, Generics, Ident, Type, TypePath, Variant, WhereClause,
    parse_quote,
};

// MARK: Expansion

/// Expand `#[derive(Event)]` for `input`.
///
/// Fails if `input` is not an enum, if a variant does not hold exactly one
/// unnamed field, if two variants hold the same type, or if a variant holds a
/// bare type parameter of the enum.
pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let variants = collect_variants(&input)?;
    let mut output = TokenStream::new();

    for variant in &variants {
        output.extend(from_impl(&input, variant));
        output.extend(variant_impl(&input, variant, variants.len()));
    }

    output.extend(sub_event_impl(&input, &variants));

    Ok(output)
}

// MARK: Variants

/// A variant `name(payload)` of the event enum.
struct EventVariant<'a> {
    name: &'a Ident,
    payload: &'a Type,
}

/// Validate `input` and collect its variants.
fn collect_variants(input: &DeriveInput) -> syn::Result<Vec<EventVariant<'_>>> {
    let data = match &input.data {
        Data::Enum(data) => data,
        Data::Struct(data) => return Err(not_an_enum(&data.struct_token)),
        Data::Union(data) => return Err(not_an_enum(&data.union_token)),
    };

    let mut variants = Vec::with_capacity(data.variants.len());
    let mut seen_payloads = HashSet::new();

    for variant in &data.variants {
        variants.push(collect_variant(
            &input.generics,
            variant,
            &mut seen_payloads,
        )?);
    }

    Ok(variants)
}

fn not_an_enum(keyword: &impl ToTokens) -> syn::Error {
    syn::Error::new_spanned(keyword, "`Event` can only be derived for enums")
}

fn collect_variant<'a>(
    generics: &Generics,
    variant: &'a Variant,
    seen_payloads: &mut HashSet<String>,
) -> syn::Result<EventVariant<'a>> {
    let payload = single_unnamed_field(variant)?;

    // A bare type parameter can be instantiated with the type of any other
    // variant, which would make the generated impls overlap in a way the
    // duplicate check below can not see.
    if is_type_parameter(payload, generics) {
        return Err(syn::Error::new_spanned(
            payload,
            "event variants can not hold a bare type parameter",
        ));
    }

    // Aliases are not resolved here: two paths to one type are caught by the
    // compiler as conflicting implementations of the expanded impls.
    if !seen_payloads.insert(type_key(payload)) {
        return Err(syn::Error::new_spanned(
            payload,
            format!("duplicate event type `{}`", type_display(payload)),
        ));
    }

    Ok(EventVariant {
        name: &variant.ident,
        payload,
    })
}

fn single_unnamed_field(variant: &Variant) -> syn::Result<&Type> {
    if let Fields::Unnamed(fields) = &variant.fields {
        let mut fields = fields.unnamed.iter();

        if let (Some(field), None) = (fields.next(), fields.next()) {
            return Ok(&field.ty);
        }
    }

    Err(syn::Error::new_spanned(
        variant,
        "event variants must hold exactly one unnamed field, as in `Variant(Payload)`",
    ))
}

fn is_type_parameter(payload: &Type, generics: &Generics) -> bool {
    let Type::Path(TypePath { qself: None, path }) = payload else {
        return false;
    };

    path.get_ident()
        .is_some_and(|ident| generics.type_params().any(|param| param.ident == *ident))
}

/// Identify a type by its tokens, ignoring whitespace.
fn type_key(payload: &Type) -> String {
    let mut key = payload.to_token_stream().to_string();

    key.retain(|character| !character.is_whitespace());

    key
}

/// Render `ty` the way it is written in source code.
///
/// Token streams print a space between every pair of tokens, as in
/// `Vec < u8 >`. Removing the spaces around the punctuation that is written
/// tight in practice keeps the diagnostics readable.
fn type_display(ty: &Type) -> String {
    ty.to_token_stream()
        .to_string()
        .replace(" :: ", "::")
        .replace(" < ", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace("& ", "&")
        .replace(" ,", ",")
}

// MARK: From

fn from_impl(input: &DeriveInput, variant: &EventVariant<'_>) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let enum_name = &input.ident;
    let EventVariant { name, payload } = variant;

    quote! {
        impl #impl_generics ::core::convert::From<#payload> for #enum_name #type_generics #where_clause {
            fn from(value: #payload) -> Self {
                Self::#name(value)
            }
        }
    }
}

// MARK: Variant

fn variant_impl(input: &DeriveInput, variant: &EventVariant<'_>, count: usize) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let enum_name = &input.ident;
    let EventVariant { name, payload } = variant;

    // With a single variant the fallback arms would be unreachable patterns.
    let into_fallback = (count > 1).then(|| quote!(other => ::core::result::Result::Err(other),));
    let ref_fallback = (count > 1).then(|| quote!(_ => ::core::option::Option::None,));

    quote! {
        impl #impl_generics ::sotu::event::Variant<#payload> for #enum_name #type_generics #where_clause {
            fn from_variant(value: #payload) -> Self {
                Self::#name(value)
            }

            fn into_variant(self) -> ::core::result::Result<#payload, Self> {
                match self {
                    Self::#name(value) => ::core::result::Result::Ok(value),
                    #into_fallback
                }
            }

            fn as_variant_ref(&self) -> ::core::option::Option<&#payload> {
                match self {
                    Self::#name(value) => ::core::option::Option::Some(value),
                    #ref_fallback
                }
            }
        }
    }
}

// MARK: SubEvent

fn sub_event_impl(input: &DeriveInput, variants: &[EventVariant<'_>]) -> TokenStream {
    let enum_name = &input.ident;
    let parent = fresh_parent_ident(input);

    let bounds: Vec<TokenStream> = variants
        .iter()
        .map(|variant| {
            let payload = variant.payload;

            quote!(#parent: ::sotu::event::Variant<#payload>)
        })
        .collect();

    let generics = sub_event_generics(&input.generics, &parent, &bounds);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, type_generics, _) = input.generics.split_for_impl();

    let accepts = sub_event_accepts(&parent, variants);
    let project = sub_event_project(&parent, variants);
    let embed = sub_event_embed(&parent, variants);

    quote! {
        impl #impl_generics ::sotu::event::SubEvent<#parent> for #enum_name #type_generics #where_clause {
            #accepts

            #project

            #embed
        }
    }
}

fn sub_event_accepts(parent: &Ident, variants: &[EventVariant<'_>]) -> TokenStream {
    if variants.is_empty() {
        return quote! {
            fn accepts(_parent: &#parent) -> bool {
                false
            }
        };
    }

    let checks = variants.iter().map(|variant| {
        let payload = variant.payload;

        quote! {
            <#parent as ::sotu::event::Variant<#payload>>::as_variant_ref(parent).is_some()
        }
    });

    quote! {
        fn accepts(parent: &#parent) -> bool {
            #(#checks)||*
        }
    }
}

fn sub_event_project(parent: &Ident, variants: &[EventVariant<'_>]) -> TokenStream {
    // Each attempt moves the parent into `into_variant` and takes it back from
    // the `Err`, so the parent is never cloned and a rejection returns it
    // unchanged.
    let attempts = variants.iter().map(|variant| {
        let EventVariant { name, payload } = variant;

        quote! {
            let parent = match <#parent as ::sotu::event::Variant<#payload>>::into_variant(parent) {
                ::core::result::Result::Ok(value) => {
                    return ::core::result::Result::Ok(Self::#name(value));
                }
                ::core::result::Result::Err(parent) => parent,
            };
        }
    });

    quote! {
        fn project(parent: #parent) -> ::core::result::Result<Self, #parent> {
            #(#attempts)*

            ::core::result::Result::Err(parent)
        }
    }
}

fn sub_event_embed(parent: &Ident, variants: &[EventVariant<'_>]) -> TokenStream {
    let arms = variants.iter().map(|variant| {
        let EventVariant { name, payload } = variant;

        quote! {
            Self::#name(value) => <#parent as ::sotu::event::Variant<#payload>>::from_variant(value),
        }
    });

    quote! {
        fn embed(self) -> #parent {
            match self {
                #(#arms)*
            }
        }
    }
}

/// Add the parent type parameter and its bounds to a copy of `generics`.
///
/// The parent parameter follows the lifetime parameters, as the language
/// requires, and its bounds precede the enum's own where-predicates.
fn sub_event_generics(generics: &Generics, parent: &Ident, bounds: &[TokenStream]) -> Generics {
    let mut generics = generics.clone();

    let lifetime_count = generics
        .params
        .iter()
        .take_while(|param| matches!(param, GenericParam::Lifetime(_)))
        .count();

    let mut params: Vec<GenericParam> = generics.params.into_iter().collect();

    params.insert(lifetime_count, parse_quote!(#parent));
    generics.params = params.into_iter().collect();

    let mut predicates = Punctuated::new();

    for bound in bounds {
        predicates.push(parse_quote!(#bound));
    }

    if let Some(existing) = generics.where_clause.take() {
        predicates.extend(existing.predicates);
    }

    generics.where_clause = Some(WhereClause {
        where_token: Default::default(),
        predicates,
    });

    generics
}

/// Pick a parent type parameter name that no token of the enum uses.
///
/// Checking every token, not only the generic parameters, keeps a payload type
/// that happens to share the name from being shadowed by the parameter.
fn fresh_parent_ident(input: &DeriveInput) -> Ident {
    let mut taken = Vec::new();

    collect_idents(input.to_token_stream(), &mut taken);

    let mut name = String::from("SotuParentEvent");

    while taken.contains(&name) {
        name.push('_');
    }

    Ident::new(&name, Span::call_site())
}

fn collect_idents(tokens: TokenStream, idents: &mut Vec<String>) {
    for token in tokens {
        match token {
            TokenTree::Ident(ident) => idents.push(ident.to_string()),
            TokenTree::Group(group) => collect_idents(group.stream(), idents),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

// MARK: Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_display_renders_types_as_written() {
        let cases: [(Type, &str); 6] = [
            (parse_quote!(Tick), "Tick"),
            (parse_quote!(common::Tick), "common::Tick"),
            (parse_quote!(Vec<u8>), "Vec<u8>"),
            (parse_quote!(&'a str), "&'a str"),
            (parse_quote!((u8, u16)), "(u8, u16)"),
            (
                parse_quote!(Result<Vec<u8>, &'a mut str>),
                "Result<Vec<u8>, &'a mut str>",
            ),
        ];

        for (payload, expected) in cases {
            assert_eq!(type_display(&payload), expected);
        }
    }
}
