use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, Ident, ItemImpl, LitStr, Path, Token};

const HANDLERS: &[&str] = &[
    "on_will_appear",
    "on_will_disappear",
    "on_did_receive_settings",
    "on_settings_changed",
    "on_property_inspector_did_appear",
    "on_property_inspector_did_disappear",
    "on_property_inspector_message",
    "on_title_parameters_did_change",
    "on_did_receive_resources",
    "on_key_down",
    "on_key_up",
    "on_dial_down",
    "on_dial_up",
    "on_dial_rotate",
    "on_touch_tap",
];

struct ActionArgs {
    uuid: LitStr,
    settings: Path,
    state: Path,
    controller: Ident,
}

impl Parse for ActionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut uuid = None;
        let mut settings = None;
        let mut state = None;
        let mut controller = format_ident!("Keypad");

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "uuid" => {
                    uuid = Some(input.parse()?);
                }
                "settings" => {
                    settings = Some(input.parse()?);
                }
                "state" => {
                    state = Some(input.parse()?);
                }
                "controller" => {
                    controller = input.parse()?;
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown streamdeck_action argument `{other}`"),
                    ));
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            uuid: uuid.ok_or_else(|| syn::Error::new(input.span(), "missing `uuid = \"...\"`"))?,
            settings: settings
                .ok_or_else(|| syn::Error::new(input.span(), "missing `settings = Type`"))?,
            state: state.ok_or_else(|| syn::Error::new(input.span(), "missing `state = Type`"))?,
            controller,
        })
    }
}

/// Fills in `Action`, registers the UUID, and keeps handwritten handlers.
#[proc_macro_attribute]
pub fn streamdeck_action(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ActionArgs);
    let impl_block = parse_macro_input!(item as ItemImpl);
    expand(args, impl_block)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

fn expand(args: ActionArgs, mut impl_block: ItemImpl) -> syn::Result<proc_macro2::TokenStream> {
    let ActionArgs {
        uuid,
        settings,
        state,
        controller,
    } = args;
    let self_ty = &impl_block.self_ty;

    let mut handler_impls = Vec::new();
    for item in &mut impl_block.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        let name = method.sig.ident.to_string();
        if !HANDLERS.contains(&name.as_str()) {
            continue;
        }
        rewrite_action_context(&mut method.sig, &settings, &state);
        let sig = &method.sig;
        let ident = &method.sig.ident;
        let args: Vec<_> = method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat) => Some(&pat.pat),
            })
            .collect();
        handler_impls.push(quote! {
            #sig {
                Self::#ident(self, #(#args),*).await
            }
        });
    }

    Ok(quote! {
        #impl_block

        #[::streamdeck_plugin::async_trait]
        impl ::streamdeck_plugin::Action for #self_ty {
            type Settings = #settings;
            type State = #state;
            const UUID: &'static str = #uuid;

            fn new(_state: ::std::sync::Arc<Self::State>) -> Self {
                <Self as ::std::default::Default>::default()
            }

            #(#handler_impls)*
        }

        ::streamdeck_plugin::inventory::submit! {
            ::streamdeck_plugin::ActionRegistration {
                uuid: #uuid,
                controller: ::streamdeck_plugin::Controller::#controller,
                factory: |state: ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>| {
                    let state = state
                        .downcast::<#state>()
                        .expect("plugin state type does not match this action's State");
                    Box::new(::streamdeck_plugin::TypedInstance::<#self_ty>::new(state))
                },
            }
        }
    })
}

fn rewrite_action_context(sig: &mut syn::Signature, settings: &Path, state: &Path) {
    for input in &mut sig.inputs {
        let syn::FnArg::Typed(typed) = input else {
            continue;
        };
        let syn::Type::Reference(reference) = typed.ty.as_mut() else {
            continue;
        };
        let syn::Type::Path(path) = reference.elem.as_mut() else {
            continue;
        };
        let Some(last) = path.path.segments.last_mut() else {
            continue;
        };
        if last.ident != "ActionContext" {
            continue;
        }
        last.arguments = syn::PathArguments::AngleBracketed(syn::parse_quote! {
            <'_, #settings, #state>
        });
    }
}

/// Accepts unused `Expr` so `controller = Keypad` stays an ident, not an expr parse fallback.
#[allow(dead_code)]
fn _assert_expr(_expr: Expr) {}
