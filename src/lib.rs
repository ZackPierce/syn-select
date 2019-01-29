//! Library to get a specific element by path. For more information on the
//!

use syn::Item;

mod error;
mod search;
mod selector;

pub use error::Error;
pub(crate) use selector::Selector;

/// Parse a path, then search a file for all results that exactly match the specified
/// path.
///
/// # Returns
/// This function can find multiple items if:
///
/// 1. There is a module and a function of the same name
/// 2. The same path is declared multiple times, differing by config flags
pub fn select(path: &str, file: &syn::File) -> Result<Vec<Item>, Error> {
    Ok(path.parse::<Selector>()?.search(file))
}

#[cfg(test)]
mod tests {
    use syn::Item;

    use super::select;

    fn sample() -> syn::File {
        syn::parse_str(
            "mod a {
            mod b {
                trait C {
                    fn d() {
                        struct E;
                    }
                    fn f(self) {}
                }
            }
            fn b() {}
        }",
        )
        .unwrap()
    }

    fn sample_with_cfg() -> syn::File {
        syn::parse_str(
            r#"
            #[cfg(feature = "g")]
            mod imp {
                pub struct H(u8);
            }
            #[cfg(not(feature = "g"))]
            mod imp {
                pub struct H(u16);
            }"#,
        )
        .unwrap()
    }

    fn search_sample(path: &str) -> Vec<syn::Item> {
        select(path, &sample()).unwrap()
    }

    fn ident(ident: &str) -> syn::Ident {
        syn::parse_str::<syn::Ident>(ident).unwrap()
    }

    #[test]
    fn example_1() {
        let result = search_sample("a::b::C");
        assert_eq!(result.len(), 1);
        if let Item::Trait(item) = &result[0] {
            assert_eq!(item.ident, ident("C"));
        } else {
            panic!("Result was wrong type {:?}", &result[0]);
        }
    }

    #[test]
    fn example_2() {
        let result = search_sample("a::b::C::d::E");
        assert_eq!(result.len(), 1);
        if let Item::Struct(item) = &result[0] {
            assert_eq!(item.ident, ident("E"));
        } else {
            panic!("Result was wrong type {:?}", &result[0]);
        }
    }

    /// If I query for "a::b::C::f" I should get the trait C filtered down to only function f.
    /// The trait needs to be included because fn f(self) {} by itself is not a valid top-level
    /// Item.
    #[test]
    fn example_3() {
        let result = search_sample("a::b::C::f");
        assert_eq!(result.len(), 1);
        if let Item::Trait(item) = &result[0] {
            assert_eq!(item.items.len(), 1);
            if let syn::TraitItem::Method(item) = &item.items[0] {
                assert_eq!(item.sig.ident, ident("f"));
            }
        }
    }

    #[test]
    fn example_4() {
        let result = search_sample("a::b");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn example_5() {
        let result = select("imp::H", &sample_with_cfg()).unwrap();
        assert_eq!(result.len(), 2);
        if let Item::Struct(item) = &result[0] {
            assert_eq!(item.attrs[0].path, syn::parse_str("cfg").unwrap());
        } else {
            panic!("First result should be struct");
        }
    }
}