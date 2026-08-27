use yew::prelude::*;

use crate::{
    browser::{self, LOCALE_STORAGE_KEY},
    models::UiMessage,
};

mod de;
mod en;
mod la;
mod zh;

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
            Locale::De => de::lookup(key),
            Locale::En => en::lookup(key),
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
        match self.locale.lookup(key) {
            Some(value) => value.to_string(),
            None => key.to_string(),
        }
    }

    pub fn translate_params(&self, key: &str, params: &[(&str, &str)]) -> String {
        let template = self
            .locale
            .lookup(key)
            .map(str::to_string)
            .unwrap_or_else(|| key.to_string());

        render_template(template, params.iter().copied())
    }

    pub fn translate_message(&self, message: &UiMessage) -> String {
        let template = self
            .locale
            .lookup(&message.key)
            .map(str::to_string)
            .unwrap_or_else(|| message.key.clone());

        render_template(
            template,
            message
                .params
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str())),
        )
    }

    #[cfg(test)]
    pub fn test_default() -> Self {
        Self {
            locale: Locale::En,
            set_locale: Callback::noop(),
        }
    }
}

#[hook]
pub fn use_i18n() -> I18n {
    use_context::<I18n>().expect("I18nProvider missing from component tree")
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
            persist_locale(next);
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
    if let Some(locale) = browser::hash_param("lang").and_then(|code| Locale::from_code(&code)) {
        persist_locale(locale);
        return Some(locale);
    }

    if let Some(stored) = browser::local_storage()
        .and_then(|storage| storage.get_item(LOCALE_STORAGE_KEY).ok().flatten())
        .and_then(|code| Locale::from_code(&code))
    {
        return Some(stored);
    }

    let navigator = web_sys::window()?.navigator();
    let tag = navigator.language()?;
    let locale = Locale::from_bcp47(&tag)?;
    persist_locale(locale);
    Some(locale)
}

fn persist_locale(locale: Locale) {
    if let Some(storage) = browser::local_storage() {
        let _ = storage.set_item(LOCALE_STORAGE_KEY, locale.code());
    }
}

fn render_template<'a>(
    template: String,
    params: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> String {
    params.into_iter().fold(template, |output, (key, value)| {
        output.replace(&format!("{{{key}}}"), value)
    })
}

#[macro_export]
macro_rules! t {
    ($i18n:expr, $key:literal) => {
        $i18n.t($key)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bcp47_tags() {
        assert_eq!(Locale::from_bcp47("en-US"), Some(Locale::En));
        assert_eq!(Locale::from_bcp47("de-DE"), Some(Locale::De));
        assert_eq!(Locale::from_bcp47("zh_CN"), Some(Locale::Zh));
        assert_eq!(Locale::from_bcp47("fr"), None);
    }

    #[test]
    fn falls_back_to_english_when_locale_missing_key() {
        if let Some(only_in_en) = en::TABLE
            .iter()
            .find_map(|(key, _)| zh::lookup(key).is_none().then_some(*key))
        {
            assert_eq!(Locale::Zh.lookup(only_in_en), Locale::En.lookup(only_in_en));
        }
    }

    #[test]
    fn locale_tables_stay_in_sync() {
        let en_keys = en::TABLE.iter().map(|(key, _)| *key).collect::<Vec<_>>();
        let de_keys = de::TABLE.iter().map(|(key, _)| *key).collect::<Vec<_>>();
        let la_keys = la::TABLE.iter().map(|(key, _)| *key).collect::<Vec<_>>();
        let zh_keys = zh::TABLE.iter().map(|(key, _)| *key).collect::<Vec<_>>();
        assert_eq!(de_keys, en_keys);
        assert_eq!(la_keys, en_keys);
        assert_eq!(zh_keys, en_keys);
    }

    #[test]
    fn returns_key_when_translation_missing() {
        let i18n = I18n {
            locale: Locale::En,
            set_locale: Callback::noop(),
        };
        assert_eq!(i18n.t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn translates_message_with_params() {
        let i18n = I18n::test_default();
        let message = UiMessage::key("sidebar.session_authed_template")
            .with_param("mnt", "EXAMPLE-MNT")
            .with_param("label", "Registry SSH");
        assert_eq!(
            i18n.translate_message(&message),
            "You authenticated as EXAMPLE-MNT via Registry SSH."
        );
    }

    #[test]
    fn translates_string_with_params() {
        let i18n = I18n {
            locale: Locale::Zh,
            set_locale: Callback::noop(),
        };

        assert_eq!(
            i18n.translate_params("step.auth_redirect.link", &[("provider", "GitHub")]),
            "前往 dn42-auth.owo.li 登录"
        );
    }

    #[test]
    fn falls_back_to_message_key_when_translation_is_missing() {
        let i18n = I18n::test_default();
        let message = UiMessage::raw("Raw fallback");
        assert_eq!(i18n.translate_message(&message), "Raw fallback");
    }
}
