use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use super::base::{
    display_path_preferring_local, synced_json_mtime, synced_json_path, synced_json_save,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ColorStyles {
    pub use_custom_colors: bool,
    pub bg: String,
    pub panel: String,
    pub surface: String,
    pub elevated: String,
    pub border: String,
    pub border_hover: String,
    pub fg: String,
    pub secondary: String,
    pub muted: String,
    pub accent: String,
    pub accent_bg: String,
    pub hover: String,
    pub tab_active: String,
    pub tab_inactive: String,
}

impl Default for ColorStyles {
    fn default() -> Self {
        Self {
            use_custom_colors: false,
            bg: "#19191c".to_string(),
            panel: "#29292e".to_string(),
            surface: "#212126".to_string(),
            elevated: "#35353c".to_string(),
            border: "#37373f".to_string(),
            border_hover: "#4b4b56".to_string(),
            fg: "#f0f0f3".to_string(),
            secondary: "#c2c2cb".to_string(),
            muted: "#8a8a95".to_string(),
            accent: "#7a88f7".to_string(),
            accent_bg: "#262942".to_string(),
            hover: "#33333a".to_string(),
            tab_active: "#29292e".to_string(),
            tab_inactive: "transparent".to_string(),
        }
    }
}

fn default_topbar_layout() -> String {
    "deck".to_string()
}

fn default_bookmarks_bar() -> String {
    "always".to_string()
}

fn default_titlebar_height() -> String {
    "40px".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TopbarStyles {
    pub visible_on_desktop: bool,
    pub visible_on_mobile: bool,
    #[serde(default = "default_topbar_layout")]
    pub layout: String,
    #[serde(default = "default_bookmarks_bar")]
    pub bookmarks_bar: String,
    pub height: String,
    #[serde(default = "default_titlebar_height")]
    pub titlebar_height: String,
    pub padding_x: String,
    pub background: String,
    pub border_bottom: String,
    pub button_size: String,
    pub button_radius: String,
    pub custom_css: String,
}

impl Default for TopbarStyles {
    fn default() -> Self {
        Self {
            visible_on_desktop: true,
            visible_on_mobile: false,
            layout: "deck".to_string(),
            bookmarks_bar: "always".to_string(),
            height: "38px".to_string(),
            titlebar_height: "40px".to_string(),
            padding_x: "6px".to_string(),
            background: "var(--fnx-panel)".to_string(),
            border_bottom: "1px solid var(--fnx-border)".to_string(),
            button_size: "28px".to_string(),
            button_radius: "6px".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TabStyles {
    pub height: String,
    pub width: String,
    pub min_width: String,
    pub gap: String,
    pub border_radius: String,
    pub font_size: String,
    pub show_indicator_dot: bool,
    pub close_on_hover_only: bool,
    pub custom_css: String,
}

impl Default for TabStyles {
    fn default() -> Self {
        Self {
            height: "36px".to_string(),
            width: "240px".to_string(),
            min_width: "120px".to_string(),
            gap: "0px".to_string(),
            border_radius: "6px".to_string(),
            font_size: "12px".to_string(),
            show_indicator_dot: true,
            close_on_hover_only: false,
            custom_css: String::new(),
        }
    }
}

fn default_addr_mode() -> String {
    "inline".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AddressBarStyles {
    #[serde(default = "default_addr_mode")]
    pub mode: String,
    pub position: String,
    pub height: String,
    pub width: String,
    pub border_radius: String,
    pub font_size: String,
    pub background: String,
    pub border: String,
    pub custom_css: String,
}

impl Default for AddressBarStyles {
    fn default() -> Self {
        Self {
            mode: "inline".to_string(),
            position: "center".to_string(),
            height: "30px".to_string(),
            width: "340px".to_string(),
            border_radius: "9px".to_string(),
            font_size: "12.5px".to_string(),
            background: "var(--fnx-bg)".to_string(),
            border: "1px solid var(--fnx-border)".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SearchModalStyles {
    pub width: String,
    pub top_offset: String,
    pub border_radius: String,
    pub input_font_size: String,
    pub backdrop_color: String,
    pub background: String,
    pub border: String,
    pub custom_css: String,
}

impl Default for SearchModalStyles {
    fn default() -> Self {
        Self {
            width: "580px".to_string(),
            top_offset: "58px".to_string(),
            border_radius: "12px".to_string(),
            input_font_size: "14.5px".to_string(),
            backdrop_color: "rgba(0, 0, 0, 0.56)".to_string(),
            background: "var(--fnx-panel)".to_string(),
            border: "1px solid var(--fnx-border-hover)".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct MobileBarStyles {
    pub visible: bool,
    pub position: String,
    pub height: String,
    pub padding_x: String,
    pub background: String,
    pub border: String,
    pub pill_height: String,
    pub pill_radius: String,
    pub button_size: String,
    pub auto_hide_on_keyboard: bool,
    pub custom_css: String,
}

impl Default for MobileBarStyles {
    fn default() -> Self {
        Self {
            visible: true,
            position: "bottom".to_string(),
            height: "54px".to_string(),
            padding_x: "8px".to_string(),
            background: "var(--fnx-panel)".to_string(),
            border: "1px solid var(--fnx-border)".to_string(),
            pill_height: "38px".to_string(),
            pill_radius: "11px".to_string(),
            button_size: "38px".to_string(),
            auto_hide_on_keyboard: true,
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TabDrawerStyles {
    pub max_height: String,
    pub border_radius: String,
    pub card_radius: String,
    pub card_padding: String,
    pub custom_css: String,
}

impl Default for TabDrawerStyles {
    fn default() -> Self {
        Self {
            max_height: "80vh".to_string(),
            border_radius: "16px".to_string(),
            card_radius: "10px".to_string(),
            card_padding: "11px 13px".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SettingsPageStyles {
    pub full_width: bool,
    pub sidebar_width: String,
    pub content_padding: String,
    pub card_radius: String,
    pub custom_css: String,
}

impl Default for SettingsPageStyles {
    fn default() -> Self {
        Self {
            full_width: true,
            sidebar_width: "244px".to_string(),
            content_padding: "32px 40px".to_string(),
            card_radius: "10px".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct NewtabStyles {
    pub title: String,
    pub subtitle: String,
    pub show_clock: bool,
    pub use_24h_clock: bool,
    pub show_shortcuts: bool,
    /// `0` = responsive auto-fit grid; `2..=6` = fixed column count on desktop
    pub shortcuts_columns: u8,
    pub show_history: bool,
    pub custom_css: String,
}

impl Default for NewtabStyles {
    fn default() -> Self {
        Self {
            title: "Fynexo".to_string(),
            subtitle: "Single-WebView studio browser · Zero-FX compositing".to_string(),
            show_clock: true,
            use_24h_clock: true,
            show_shortcuts: true,
            shortcuts_columns: 0,
            show_history: true,
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct ComponentStyles {
    pub colors: ColorStyles,
    pub topbar: TopbarStyles,
    pub tabs: TabStyles,
    pub address_bar: AddressBarStyles,
    pub search_modal: SearchModalStyles,
    pub mobile_bar: MobileBarStyles,
    pub tab_drawer: TabDrawerStyles,
    pub settings_page: SettingsPageStyles,
    pub newtab: NewtabStyles,
}

impl ComponentStyles {
    pub fn styles_path() -> PathBuf {
        synced_json_path("fynexo_styles.json")
    }

    pub fn styles_path_display() -> String {
        display_path_preferring_local("fynexo_styles.json")
    }

    pub fn load() -> Self {
        let path = Self::styles_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(styles) = serde_json::from_str::<ComponentStyles>(&content) {
                    return styles;
                }
            }
        }
        let default_styles = Self::default();
        default_styles.save();
        default_styles
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            synced_json_save("fynexo_styles.json", &json);
        }
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("fynexo_styles.json")
    }
}
