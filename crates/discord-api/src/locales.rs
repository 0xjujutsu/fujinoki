use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::{
    tasks as turbo_tasks,
    tasks::{RcStr, TaskInput, Vc},
};

lazy_static! {
    pub static ref LOCALES: Vc<Locales> = Locales {
        id: Locale {
            name: "Indonesian".into(),
            native_name: "Bahasa Indonesia".into(),
        },
        da: Locale {
            name: "Danish".into(),
            native_name: "Dansk".into(),
        },
        de: Locale {
            name: "German".into(),
            native_name: "Deutsch".into(),
        },
        en_gb: Locale {
            name: "English, UK".into(),
            native_name: "English, UK".into(),
        },
        en_us: Locale {
            name: "English, US".into(),
            native_name: "English, US".into(),
        },
        es_es: Locale {
            name: "Spanish".into(),
            native_name: "Español".into(),
        },
        fr: Locale {
            name: "French".into(),
            native_name: "Français".into(),
        },
        hr: Locale {
            name: "Croatian".into(),
            native_name: "Hrvatski".into(),
        },
        it: Locale {
            name: "Italian".into(),
            native_name: "Italiano".into(),
        },
        lt: Locale {
            name: "Lithuanian".into(),
            native_name: "Lietuviškai".into(),
        },
        hu: Locale {
            name: "Hungarian".into(),
            native_name: "Magyar".into(),
        },
        nl: Locale {
            name: "Dutch".into(),
            native_name: "Nederlands".into(),
        },
        no: Locale {
            name: "Norwegian".into(),
            native_name: "Norsk".into(),
        },
        pl: Locale {
            name: "Polish".into(),
            native_name: "Polski".into(),
        },
        pt_br: Locale {
            name: "Portuguese, Brazilian".into(),
            native_name: "Português do Brasil".into(),
        },
        ro: Locale {
            name: "Romanian, Romania".into(),
            native_name: "Română".into(),
        },
        fi: Locale {
            name: "Finnish".into(),
            native_name: "Suomi".into(),
        },
        sv_se: Locale {
            name: "Swedish".into(),
            native_name: "Svenska".into(),
        },
        vi: Locale {
            name: "Vietnamese".into(),
            native_name: "Tiếng Việt".into(),
        },
        tr: Locale {
            name: "Turkish".into(),
            native_name: "Türkçe".into(),
        },
        cs: Locale {
            name: "Czech".into(),
            native_name: "Čeština".into(),
        },
        el: Locale {
            name: "Greek".into(),
            native_name: "Ελληνικά".into(),
        },
        bg: Locale {
            name: "Bulgarian".into(),
            native_name: "български".into(),
        },
        ru: Locale {
            name: "Russian".into(),
            native_name: "Pусский".into(),
        },
        uk: Locale {
            name: "Ukrainian".into(),
            native_name: "Українська".into(),
        },
        hi: Locale {
            name: "Hindi".into(),
            native_name: "हिन्दी".into(),
        },
        th: Locale {
            name: "Thai".into(),
            native_name: "ไทย".into(),
        },
        zh_cn: Locale {
            name: "Chinese, China".into(),
            native_name: "中文".into(),
        },
        ja: Locale {
            name: "Japanese".into(),
            native_name: "日本語".into(),
        },
        zh_tw: Locale {
            name: "Chinese, Taiwan".into(),
            native_name: "繁體中文".into(),
        },
        ko: Locale {
            name: "Korean".into(),
            native_name: "한국어".into(),
        },
    }
    .cell();
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct Locales {
    pub id: Locale,
    pub da: Locale,
    pub de: Locale,
    pub en_gb: Locale,
    pub en_us: Locale,
    pub es_es: Locale,
    pub fr: Locale,
    pub hr: Locale,
    pub it: Locale,
    pub lt: Locale,
    pub hu: Locale,
    pub nl: Locale,
    pub no: Locale,
    pub pl: Locale,
    pub pt_br: Locale,
    pub ro: Locale,
    pub fi: Locale,
    pub sv_se: Locale,
    pub vi: Locale,
    pub tr: Locale,
    pub cs: Locale,
    pub el: Locale,
    pub bg: Locale,
    pub ru: Locale,
    pub uk: Locale,
    pub hi: Locale,
    pub th: Locale,
    pub zh_cn: Locale,
    pub ja: Locale,
    pub zh_tw: Locale,
    pub ko: Locale,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct Locale {
    pub name: RcStr,
    pub native_name: RcStr,
}
