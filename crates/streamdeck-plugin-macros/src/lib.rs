use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote, Ident, ImplItem, ItemImpl, LitStr, Path, Token};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionKind {
    Keypad,
    Encoder,
    Action,
}

struct ActionArgs {
    uuid: LitStr,
    settings: Path,
    state: Path,
    controller: Option<Ident>,
}

impl Parse for ActionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut uuid = None;
        let mut settings = None;
        let mut state = None;
        let mut controller = None;

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
                    controller = Some(input.parse()?);
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

/// Fills in associated types, implements [`Action`], and registers the UUID.
///
/// Apply to `impl KeypadAction for T`, `impl EncoderAction for T`, or
/// `impl Action for T` so rust-analyzer can complete handler names.
#[proc_macro_attribute]
pub fn streamdeck_action(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ActionArgs);
    let impl_block = parse_macro_input!(item as ItemImpl);
    expand(args, impl_block)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

fn expand(args: ActionArgs, mut impl_block: ItemImpl) -> syn::Result<proc_macro2::TokenStream> {
    let Some((_, trait_path, _)) = &impl_block.trait_ else {
        return Err(syn::Error::new_spanned(
            &impl_block.self_ty,
            "#[streamdeck_action] expects `impl KeypadAction for YourType`, `impl EncoderAction for YourType`, or `impl Action for YourType`",
        ));
    };
    let trait_name = trait_path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(trait_path, "missing trait name"))?
        .ident
        .to_string();
    let kind = match trait_name.as_str() {
        "KeypadAction" => ActionKind::Keypad,
        "EncoderAction" => ActionKind::Encoder,
        "Action" => ActionKind::Action,
        other => {
            return Err(syn::Error::new_spanned(
                trait_path,
                format!(
                    "#[streamdeck_action] does not support `{other}`; use KeypadAction, EncoderAction, or Action"
                ),
            ));
        }
    };

    let ActionArgs {
        uuid,
        settings,
        state,
        controller,
    } = args;

    inject_assoc_type(&mut impl_block, "Settings", &settings);
    inject_assoc_type(&mut impl_block, "State", &state);
    if kind == ActionKind::Action {
        inject_action_defaults(&mut impl_block, &uuid);
    }

    for item in &mut impl_block.items {
        let ImplItem::Fn(method) = item else {
            continue;
        };
        rewrite_action_context(&mut method.sig, &settings, &state);
    }

    let self_ty = &impl_block.self_ty;
    let controller = match (kind, controller) {
        (_, Some(ident)) => quote!(::streamdeck_plugin::Controller::#ident),
        (ActionKind::Encoder, None) => quote!(::streamdeck_plugin::Controller::Encoder),
        (ActionKind::Keypad | ActionKind::Action, None) => {
            quote!(::streamdeck_plugin::Controller::Keypad)
        }
    };

    let action_bridge = match kind {
        ActionKind::Action => quote! {},
        ActionKind::Keypad => action_bridge(
            self_ty,
            quote!(::streamdeck_plugin::KeypadAction),
            &uuid,
            KEYPAD_HANDLERS,
        ),
        ActionKind::Encoder => action_bridge(
            self_ty,
            quote!(::streamdeck_plugin::EncoderAction),
            &uuid,
            ENCODER_HANDLERS,
        ),
    };

    Ok(quote! {
        #[::streamdeck_plugin::async_trait]
        #impl_block

        #action_bridge

        ::streamdeck_plugin::inventory::submit! {
            ::streamdeck_plugin::ActionRegistration {
                uuid: #uuid,
                controller: #controller,
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

const KEYPAD_HANDLERS: &[&str] = &[
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
];

const ENCODER_HANDLERS: &[&str] = &[
    "on_will_appear",
    "on_will_disappear",
    "on_did_receive_settings",
    "on_settings_changed",
    "on_property_inspector_did_appear",
    "on_property_inspector_did_disappear",
    "on_property_inspector_message",
    "on_title_parameters_did_change",
    "on_did_receive_resources",
    "on_dial_down",
    "on_dial_up",
    "on_dial_rotate",
    "on_touch_tap",
];

fn action_bridge(
    self_ty: &syn::Type,
    trait_path: proc_macro2::TokenStream,
    uuid: &LitStr,
    handlers: &[&str],
) -> proc_macro2::TokenStream {
    let methods = handlers.iter().map(|name| forward_method(self_ty, &trait_path, name));
    quote! {
        #[::streamdeck_plugin::async_trait]
        impl ::streamdeck_plugin::Action for #self_ty {
            type Settings = <#self_ty as #trait_path>::Settings;
            type State = <#self_ty as #trait_path>::State;
            const UUID: &'static str = #uuid;

            fn new(_state: ::std::sync::Arc<Self::State>) -> Self {
                <Self as ::std::default::Default>::default()
            }

            #(#methods)*
        }
    }
}

fn forward_method(
    self_ty: &syn::Type,
    trait_path: &proc_macro2::TokenStream,
    name: &str,
) -> proc_macro2::TokenStream {
    let ident = format_ident!("{name}");
    match name {
        "on_settings_changed" => quote! {
            async fn #ident(
                &mut self,
                previous: &Self::Settings,
                ctx: &::streamdeck_plugin::ActionContext<'_, Self::Settings, Self::State>,
            ) -> ::streamdeck_plugin::Result<()> {
                <#self_ty as #trait_path>::#ident(self, previous, ctx).await
            }
        },
        "on_property_inspector_did_appear" | "on_property_inspector_did_disappear" => quote! {
            async fn #ident(
                &mut self,
                ctx: &::streamdeck_plugin::ActionContext<'_, Self::Settings, Self::State>,
            ) -> ::streamdeck_plugin::Result<()> {
                <#self_ty as #trait_path>::#ident(self, ctx).await
            }
        },
        "on_property_inspector_message" => quote! {
            async fn #ident(
                &mut self,
                payload: &::streamdeck_plugin::serde_json::Value,
                ctx: &::streamdeck_plugin::ActionContext<'_, Self::Settings, Self::State>,
            ) -> ::streamdeck_plugin::Result<()> {
                <#self_ty as #trait_path>::#ident(self, payload, ctx).await
            }
        },
        "on_title_parameters_did_change" => quote! {
            async fn #ident(
                &mut self,
                payload: &::streamdeck_plugin::TitleParametersPayload,
                ctx: &::streamdeck_plugin::ActionContext<'_, Self::Settings, Self::State>,
            ) -> ::streamdeck_plugin::Result<()> {
                <#self_ty as #trait_path>::#ident(self, payload, ctx).await
            }
        },
        "on_dial_rotate" => quote! {
            async fn #ident(
                &mut self,
                payload: &::streamdeck_plugin::DialRotatePayload,
                ctx: &::streamdeck_plugin::ActionContext<'_, Self::Settings, Self::State>,
            ) -> ::streamdeck_plugin::Result<()> {
                <#self_ty as #trait_path>::#ident(self, payload, ctx).await
            }
        },
        "on_touch_tap" => quote! {
            async fn #ident(
                &mut self,
                payload: &::streamdeck_plugin::TouchTapPayload,
                ctx: &::streamdeck_plugin::ActionContext<'_, Self::Settings, Self::State>,
            ) -> ::streamdeck_plugin::Result<()> {
                <#self_ty as #trait_path>::#ident(self, payload, ctx).await
            }
        },
        _ => quote! {
            async fn #ident(
                &mut self,
                payload: &::streamdeck_plugin::ActionPayload,
                ctx: &::streamdeck_plugin::ActionContext<'_, Self::Settings, Self::State>,
            ) -> ::streamdeck_plugin::Result<()> {
                <#self_ty as #trait_path>::#ident(self, payload, ctx).await
            }
        },
    }
}

fn inject_assoc_type(impl_block: &mut ItemImpl, name: &str, path: &Path) {
    if impl_block.items.iter().any(|item| match item {
        ImplItem::Type(ty) => ty.ident == name,
        _ => false,
    }) {
        return;
    }
    let ident = format_ident!("{name}");
    impl_block.items.insert(
        0,
        parse_quote! {
            type #ident = #path;
        },
    );
}

fn inject_action_defaults(impl_block: &mut ItemImpl, uuid: &LitStr) {
    if !impl_block.items.iter().any(|item| match item {
        ImplItem::Const(item) => item.ident == "UUID",
        _ => false,
    }) {
        impl_block.items.insert(
            0,
            parse_quote! {
                const UUID: &'static str = #uuid;
            },
        );
    }
    if !impl_block.items.iter().any(|item| match item {
        ImplItem::Fn(method) => method.sig.ident == "new",
        _ => false,
    }) {
        impl_block.items.push(parse_quote! {
            fn new(_state: ::std::sync::Arc<Self::State>) -> Self {
                <Self as ::std::default::Default>::default()
            }
        });
    }
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
