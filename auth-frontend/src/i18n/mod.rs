use gloo_storage::{LocalStorage, Storage};
use yew::prelude::*;

mod de;
mod en;
mod la;
mod zh;

const STORAGE_KEY: &str = "dn42.auth.locale";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Locale {
    #[default]
    En,
    De,
    La,
    Zh,
}

impl Locale {
    pub const ALL: &'static [Locale] = &[Locale::En, Locale::De, Locale::La, Locale::Zh];

    pub fn code(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::De => "de",
            Locale::La => "la",
            Locale::Zh => "zh",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::De => "Deutsch",
            Locale::La => "Latina",
            Locale::Zh => "中文（简体）",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Locale::En),
            "de" => Some(Locale::De),
            "la" => Some(Locale::La),
            "zh" => Some(Locale::Zh),
            _ => None,
        }
    }

    fn from_bcp47(tag: &str) -> Option<Self> {
        let primary = tag
            .split(['-', '_'])
            .next()
            .unwrap_or(tag)
            .to_ascii_lowercase();
        Self::from_code(&primary)
    }

    fn lookup(self, key: &str) -> Option<&'static str> {
        let primary = match self {
            Locale::En => en::lookup(key),
            Locale::De => de::lookup(key),
            Locale::La => la::lookup(key),
            Locale::Zh => zh::lookup(key),
        };
        primary.or_else(|| {
            if matches!(self, Locale::En) {
                None
            } else {
                en::lookup(key)
            }
        })
    }
}

#[derive(Clone, PartialEq)]
pub struct I18n {
    locale: Locale,
    set_locale: Callback<Locale>,
}

impl I18n {
    pub fn locale(&self) -> Locale {
        self.locale
    }

    pub fn set_locale(&self, locale: Locale) {
        self.set_locale.emit(locale);
    }

    pub fn t(&self, key: &'static str) -> &'static str {
        self.locale.lookup(key).unwrap_or(key)
    }

    pub fn translate_owned(&self, key: &str) -> String {
        self.locale
            .lookup(key)
            .map(str::to_string)
            .unwrap_or_else(|| key.to_string())
    }

    pub fn translate_message(&self, message: &dn42_auth_client::models::UiMessage) -> String {
        let template = self
            .locale
            .lookup(&message.key)
            .map(str::to_string)
            .unwrap_or_else(|| message.key.clone());
        message
            .params
            .iter()
            .fold(template, |output, (key, value)| {
                output.replace(&format!("{{{key}}}"), value)
            })
    }
}

#[hook]
pub fn use_i18n() -> I18n {
    use_context::<I18n>().expect("I18nProvider missing")
}

#[derive(Properties, PartialEq)]
pub struct I18nProviderProps {
    pub children: Children,
}

#[function_component(I18nProvider)]
pub fn i18n_provider(props: &I18nProviderProps) -> Html {
    let locale = use_state(|| detect_initial_locale().unwrap_or_default());

    let set_locale = {
        let locale = locale.clone();
        Callback::from(move |next: Locale| {
            let _ = LocalStorage::set(STORAGE_KEY, next.code());
            locale.set(next);
        })
    };

    let context = I18n {
        locale: *locale,
        set_locale,
    };

    html! {
        <ContextProvider<I18n> context={context}>
            { for props.children.iter() }
        </ContextProvider<I18n>>
    }
}

fn detect_initial_locale() -> Option<Locale> {
    if let Some(stored) = LocalStorage::get::<String>(STORAGE_KEY)
        .ok()
        .and_then(|code| Locale::from_code(&code))
    {
        return Some(stored);
    }

    let navigator = web_sys::window()?.navigator();
    let tag = navigator.language()?;
    Locale::from_bcp47(&tag)
}

#[cfg(test)]
mod tests {
    use dn42_auth_client::models::UiMessage;

    use super::*;

    fn test_i18n(locale: Locale) -> I18n {
        I18n {
            locale,
            set_locale: Callback::noop(),
        }
    }

    #[test]
    fn parses_bcp47_tags() {
        assert_eq!(Locale::from_bcp47("en-US"), Some(Locale::En));
        assert_eq!(Locale::from_bcp47("de-DE"), Some(Locale::De));
        assert_eq!(Locale::from_bcp47("zh_CN"), Some(Locale::Zh));
        assert_eq!(Locale::from_bcp47("fr"), None);
    }

    #[test]
    fn falls_back_to_english_when_locale_missing_key() {
        assert_eq!(
            Locale::De.lookup("error.auth.asn.required"),
            Locale::En.lookup("error.auth.asn.required")
        );
    }

    #[test]
    fn translates_backend_auth_method_messages_with_params() {
        let i18n = test_i18n(Locale::En);
        let message = UiMessage::key("auth_method.registry_pgp.description")
            .with_param("fingerprints", "0123456789ABCDEF");

        assert_eq!(
            i18n.translate_message(&message),
            "Use one of your registry PGP fingerprints: 0123456789ABCDEF"
        );
    }

    #[test]
    fn returns_key_when_translation_missing() {
        let i18n = test_i18n(Locale::En);
        assert_eq!(i18n.t("nonexistent.key"), "nonexistent.key");
    }
}
