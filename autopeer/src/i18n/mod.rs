use yew::prelude::*;

mod en;
mod zh;

const LOCALE_STORAGE_KEY: &str = "bird-lg-rs.autopeer.locale";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Locale {
    #[default]
    En,
    Zh,
}

impl Locale {
    pub const ALL: &'static [Locale] = &[Locale::En, Locale::Zh];

    pub fn code(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Zh => "zh",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Zh => "中文",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Locale::En),
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

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn detect_initial_locale() -> Option<Locale> {
    if let Some(stored) = local_storage()
        .and_then(|storage| storage.get_item(LOCALE_STORAGE_KEY).ok().flatten())
        .and_then(|code| Locale::from_code(&code))
    {
        return Some(stored);
    }

    let navigator = web_sys::window()?.navigator();
    let tag = navigator.language()?;
    Locale::from_bcp47(&tag)
}

fn persist_locale(locale: Locale) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(LOCALE_STORAGE_KEY, locale.code());
    }
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
        assert_eq!(Locale::from_bcp47("zh_CN"), Some(Locale::Zh));
        assert_eq!(Locale::from_bcp47("fr"), None);
    }

    #[test]
    fn falls_back_to_english_when_locale_missing_key() {
        let only_in_en = en::TABLE.first().expect("en table non-empty").0;
        assert_eq!(Locale::Zh.lookup(only_in_en), Locale::En.lookup(only_in_en));
    }

    #[test]
    fn returns_key_when_translation_missing() {
        let i18n = I18n {
            locale: Locale::En,
            set_locale: Callback::noop(),
        };
        assert_eq!(i18n.t("nonexistent.key"), "nonexistent.key");
    }
}
