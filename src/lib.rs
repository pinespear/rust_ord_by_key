//! Provides a convenient macro for implementing Ord trait with logic
//! specified in an inline expression
//!
//! ```
//! use ord_by_key::ord_eq_by_key_selector;
//! use core::cmp::Reverse;
//! // `Person` will be ordered by `last_name`, then by `first_name`, then by `age` in reverse
//! #[ord_eq_by_key_selector(|p|
//!     &p.last_name,
//!     &p.first_name,
//!     Reverse(p.age),)]
//! pub struct Person {
//!     pub first_name: String,
//!     pub last_name: String,
//!     pub age: usize,
//! }
//! ```
//!
//! ```
//! use ord_by_key::ord_eq_by_key_selector;
//! // Container for [`&str`] which will be ordered by underlying string length
//! #[ord_eq_by_key_selector(|(s)| s.len())]
//! pub struct StrByLen<'a> (&'a str);
//!
//! // Note, comparison happens just based on string length
//! assert!(StrByLen("Alex") > StrByLen("Bob"));
//! assert!(StrByLen("Alex") == StrByLen("John"));
//! assert!(StrByLen("Alex") < StrByLen("Michael"));
//! ```

#![deny(missing_docs)]
#![deny(warnings)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parenthesized;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token;
use syn::Expr;
use syn::Ident;
use syn::ItemStruct;
use syn::Token;

#[doc(inline)]
/// Implements [`Ord`], [`PartialOrd`], [`PartialEq`] and [`Eq`] for a struct.
///
/// Implemented comparison logic is based on a custom key extraction expression provided.
/// During comparison, key extraction expression will be evaluated for both values and then
/// resulted keys will be compared with each other. `ord_eq_by_key_selector` supports
/// multiple key expressions, which allows implement logic "sort by ..., then by ..."
///
/// # Syntax
/// ```compile_fail
/// #[ord_eq_by_key_selector(|parameter| key_expressoin, key_expressoin, ...)]
/// pub struct MyStruct {...}
///
/// #[ord_eq_by_key_selector(|(parameter, parameter, ...)| key_expressoin, key_expressoin, ...)]
/// pub struct MyStruct (...)
/// ```
/// * `parameter` - definition of the parameter or parameters which key expressions can use
/// to access underlying struct or fields within the struct. There are 2 options for defining
/// parameters:
///     * `|a|` - this syntax is similar to syntax of a regular closure definition. There can be
///       only one parameter name. Key expressions can access this parameter as a variable, it will
///       have type `&Self`. Note that you can use this option for structs either with named or unnamed
///       fields
///     * `|(a, b, c, ...)|` - this syntax can be used if underlying struct is a defined with unnamed
///       fields (e.g. `struct Hello(i32, String);`) to destruct reference to struct into a few references
///       to individual fields in the struct. Number of parameter names must match number of fields in
///       the struct.
///
/// * `key_expression` - expression which produces a key for comparison. Expression can
/// access `parameter_name` input and must return `impl Ord`. Multiple expressions can be
/// provided, comma-separated (last comma is optional). Expression can be single-line, or
/// multi-line enclosed in `{}`.
/// * `pub struct MyStruct ...` or  - definition of struct for which [`Ord`], [`PartialOrd`],
/// [`PartialEq`] and [`Eq`] will be implemented
///
/// # Comparison logic
/// Let's look at [`Ord::cmp`] implementation (rest of traits have similar implementation logic)
///
/// When 2 values `a` and `b` are compared using auto-implemented comparison method, lexicographic
/// comparison logic will be executed:
///
/// 1) use specified `key_expression`, and evaluate comparison keys for `a` and `b` (`key_a` and `key_b`)
/// 2) compare `key_a` and `key_b` using their own [`Ord`] implementaiton
/// 3) If comparisong result is not [`::core::cmp::Ordering::Equal`], return comparison reslut
/// 4) Otherwise switch to next provided key expression and redo steps 1..4 using this expresison
/// 5) If no key expressions left, return [`::core::cmp::Ordering::Equal`]
///
/// Note, all evaluations are lazy and happen only when they are necessary. For example, if first
/// provided key expression gave us non-ambigious result, then rest of key expressions will not be
/// evaluated.
///
/// # Key Expression implementation considerations and examples
///
/// In simple case, key expression returns one of fields from the underlying struct.
/// ```
/// use ord_by_key::ord_eq_by_key_selector;
/// // `Person` will be ordered by it's field `age`
/// #[ord_eq_by_key_selector(|p| p.age)]
/// pub struct Person {
///     pub first_name: String,
///     pub last_name: String,
///     pub age: usize,
/// }
/// ```
///
/// If return type does not implement [`Copy`], expression should return borrowed value
/// ```
/// use ord_by_key::ord_eq_by_key_selector;
/// // `Person` will be ordered by it's field `last_name`
/// #[ord_eq_by_key_selector(|p| &p.last_name)]
/// pub struct Person {
///     pub first_name: String,
///     pub last_name: String,
///     pub age: usize,
/// }
///
/// ```
///
/// If struct should be sorted by multiple fields, multiple expressions can be provided.
/// Note, that parameter name should be specified only once, and each of expressions can
/// use it
/// ```
/// use ord_by_key::ord_eq_by_key_selector;
/// // `Person` will be ordered by `last_name`, then by `first_name`
/// #[ord_eq_by_key_selector(|p|
///     &p.last_name,
///     &p.first_name, )]
/// pub struct Person {
///     pub first_name: String,
///     pub last_name: String,
///     pub age: usize,
/// }
/// ```
///
/// If struct should be sorted by some keys in reverse order, you can use [`::core::cmp::Reverse`]
/// container for values which should be sorted in reverse:
/// ```
/// use ord_by_key::ord_eq_by_key_selector;
/// use core::cmp::Reverse;
/// // `Person` will be ordered by `last_name`, then by `first_name`, then by `age` in reverse
/// #[ord_eq_by_key_selector(|p|
///     &p.last_name,
///     &p.first_name,
///     Reverse(p.age),)]
/// pub struct Person {
///     pub first_name: String,
///     pub last_name: String,
///     pub age: usize,
/// }
/// ```
///
/// You can use multi-line block expression and access multiple fields. You can use explicit
/// `return`
/// ```
/// use ord_by_key::ord_eq_by_key_selector;
/// // Sort Point by distance from the (0,0)
/// #[ord_eq_by_key_selector(|p|
///     {
///         let x = p.x as i64;
///         let y = p.y as i64;
///
///         // For sorting purposes, it's not necessary to take square root
///         let distance = x * x + y * y;
///         
///         return distance;
///     })]
/// pub struct Point {
///     pub x: i32,
///     pub y: i32,
/// }
/// ```
///
/// Note that all expressions are lazy evaluated every time comparison is triggered. In
/// some applications that can lead to low performance if key expressions are computationally
/// expensive and comparisons happen repeatedly.
///
/// # Custom sorting logic for existing structs
/// One of use case is introduction of custom sorting logic to existing structs or different
/// sorting logic for different cases. Example how custom logic is introduces in core library
/// is [`::core::cmp::Reverse`] container (which reverses existing comparison logic), but we can
/// implement more complicated containers.
///
/// Let's say, we want to sort integers by their absolute value, and then by the actual value.
/// We cannot introduce new sorting logic to [`i32`] because it already implements [`Ord`], but we can
/// introduce container for [`i32`] which has custom sorting logic. Note that struct is defined with
/// unnamed fields and we are using syntax to destruct it to individual fields
/// ```
/// use ord_by_key::ord_eq_by_key_selector;
/// // Container for `i32` will be ordered by absolute value, then by actual value
/// #[ord_eq_by_key_selector(|(i)| i.abs(), i)]
/// pub struct I32ByAbs(i32);
///
/// let a = I32ByAbs(10);
/// let b = I32ByAbs(-11);
///
/// assert!(a < b);
/// ```
///
/// You can use more complicated containers with generic and constraints
/// ```
/// use ord_by_key::ord_eq_by_key_selector;
/// use core::cmp::Reverse;
/// use std::collections::BinaryHeap;
/// use std::fmt::Debug;
///
/// // Merges sorted iterator and prints final sorted sequence to the console
/// fn merge_sorted_iterators<T: Ord + Debug, I: Iterator<Item = T>>(iterators: Vec<I>) {
///     let mut heap = BinaryHeap::new();
///
///     // Container for iterator and it's last value, ordered by value
///     #[ord_eq_by_key_selector(|(value, iter)| value)]
///     // Note that inner struct cannot use generic parameters from the outer function
///     // (error[E0401]) so they have to be re-defined with constraints.
///     // For the sorting container, the only constraint which we care about is `T : Ord`
///     // because we are using the value as a sorting key
///     struct ItemIter<T: Ord, I>(T, I);
///
///     for mut iterator in iterators {
///         if let Some(item) = iterator.next() {
///             // Need to Reverse here because BinaryHeap by default works as max heap
///             // and we need min heap
///             heap.push(Reverse(ItemIter(item, iterator)));
///         }
///     }
///
///     while let Some(Reverse(ItemIter(item, mut iterator))) = heap.pop() {
///         println!("{:?}", item);
///
///         if let Some(item) = iterator.next() {
///             heap.push(Reverse(ItemIter(item, iterator)));
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn ord_eq_by_key_selector(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as MacroAttribute);

    let key_selector_param = match &attr.param {
        ParamDefinition::SingleIdentifier(ident) => {
            quote! {#ident}
        }
        ParamDefinition::Tuple(tuple) => {
            quote! {
                Self (
                    #(
                        #tuple ,
                    )*
                )

            }
        }
    };

    let key_selectors = &attr.key_selectors;
    let key_selector_func_names: Vec<_> = (0..key_selectors.len())
        .map(|i| format!("_ord_eq_by_key_selector_{}", i))
        .map(|n| Ident::new(&n, proc_macro2::Span::mixed_site()))
        .collect();

    let structure = syn::parse_macro_input!(item as ItemStruct);
    let structure_name = &structure.ident;
    let generics = &structure.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let result = quote! {
        #structure

        impl #impl_generics #structure_name #ty_generics #where_clause {
            #(
                fn #key_selector_func_names  (_ord_eq_by_key_selector_do_not_use: &Self) -> impl ::core::cmp::Ord + '_ {
                    // We should allow unused variables here to avoid unnecessary warnings in case caller is
                    // using syntax |(a,b,c)| to destruct tuple type but not using all of components of the
                    // tuple in key construction
                    #[allow(unused_variables)]
                    let #key_selector_param = _ord_eq_by_key_selector_do_not_use;

                    // TODO: Re-define input parameter _ord_eq_by_key_selector_do_not_use, so key
                    // selector won't be able to do unintentional access to it (all accesses should
                    // go through user-defined parameter names)

                    #key_selectors
                }
            )*
        }

        impl #impl_generics ::core::cmp::PartialEq for #structure_name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                #(
                    let key_self = #structure_name::#key_selector_func_names(self);
                    let key_other = #structure_name::#key_selector_func_names(other);

                    let result = key_self.eq(&key_other);

                    if result != true {
                        return result;
                    }
                )*

                return true;
            }
        }

        impl #impl_generics ::core::cmp::Eq for #structure_name #ty_generics #where_clause { }

        impl #impl_generics ::core::cmp::Ord for #structure_name #ty_generics #where_clause {
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                #(
                    let key_self = #structure_name::#key_selector_func_names(self);
                    let key_other = #structure_name::#key_selector_func_names(other);

                    let result = key_self.cmp(&key_other);

                    if result != ::core::cmp::Ordering::Equal {
                        return result;
                    }
                )*

                return ::core::cmp::Ordering::Equal;
            }
        }

        impl #impl_generics ::core::cmp::PartialOrd for #structure_name #ty_generics #where_clause {
            fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::option::Option::Some(self.cmp(other))
            }
        }
    };

    result.into()
}

/// container for syntax of attribute
/// | ident | expression, expression, ....
/// There has to be at least one expression, comma-delimited
/// Last comma is optional
struct MacroAttribute {
    _bar1: Token![|],
    param: ParamDefinition,
    _bar2: Token![|],
    key_selectors: Vec<Expr>,
}

enum ParamDefinition {
    SingleIdentifier(Ident),
    Tuple(Vec<Ident>),
}

impl Parse for MacroAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MacroAttribute {
            _bar1: input.parse()?,
            param: input.parse()?,
            _bar2: input.parse()?,
            key_selectors: {
                let mut exprs = vec![];

                loop {
                    let expr: Expr = input.parse()?;
                    exprs.push(expr);

                    if input.is_empty() {
                        break;
                    }

                    let _: Token!(,) = input.parse()?;

                    if input.is_empty() {
                        break;
                    }
                }

                exprs
            },
        })
    }
}

impl Parse for ParamDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Paren) {
            let content;
            let _ = parenthesized!(content in input);

            let params: Punctuated<Ident, Token![,]> = content.parse_terminated(Ident::parse)?;

            let params: Vec<_> = params.into_iter().collect();

            return Ok(ParamDefinition::Tuple(params));
        }

        let ident: Ident = input.parse()?;
        return Ok(ParamDefinition::SingleIdentifier(ident));
    }
}
