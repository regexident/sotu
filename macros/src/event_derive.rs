use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{
    Data, DeriveInput, Fields, GenericParam, Generics, Ident, Type, WhereClause, parse_quote,
};

// MARK: Expansion

/// Expand `#[derive(Event)]` for `input`.
///
/// Fails if `input` is not an enum or if a variant does not hold exactly one
/// unnamed field.
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

fn collect_variants(input: &DeriveInput) -> syn::Result<Vec<EventVariant<'_>>> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`Event` can only be derived for enums",
        ));
    };

    let mut variants = Vec::with_capacity(data.variants.len());

    for variant in &data.variants {
        let Fields::Unnamed(fields) = &variant.fields else {
            return Err(syn::Error::new_spanned(
                variant,
                "event variants must hold exactly one unnamed field, as in `Variant(Payload)`",
            ));
        };

        let mut fields = fields.unnamed.iter();

        let (Some(field), None) = (fields.next(), fields.next()) else {
            return Err(syn::Error::new_spanned(
                variant,
                "event variants must hold exactly one unnamed field, as in `Variant(Payload)`",
            ));
        };

        variants.push(EventVariant {
            name: &variant.ident,
            payload: &field.ty,
        });
    }

    Ok(variants)
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
