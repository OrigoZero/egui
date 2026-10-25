//! The words egui's color picker draws on its own: its labels, its blending
//! choices and the hover texts of its sliders and buttons, which it puts on
//! screen without the app handing them over.
//!
//! A [`Context`] holds one [`Translations`], English until the app states
//! another with [`Context::set_translations`]. The color picker and the
//! numeric color space button read their words from the context as they
//! draw, so a change of language reaches the next pass without rebuilding
//! anything.

use std::sync::{Arc, OnceLock};

use crate::{Context, Id};

/// The words egui's built-in widgets read from their context, grouped by
/// widget.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Translations {
    /// The colour picker's popup and its button.
    pub color_picker: ColorPickerTranslations,
}

/// The words of the colour picker ([`crate::widgets::color_picker`]).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ColorPickerTranslations {
    /// Hover text of the swatch showing the colour being edited.
    pub selected_color: String,
    /// Label in front of the blending choice.
    pub blending: String,
    /// The blending choice that mixes the colour by its alpha.
    pub normal_blending: String,
    /// The blending choice that adds the colour.
    pub additive_blending: String,
    /// Hover text of the hue slider.
    pub hue: String,
    /// Hover text of the saturation slider.
    pub saturation: String,
    /// Hover text of the value slider.
    pub value: String,
    /// Hover text of the alpha slider.
    pub alpha: String,
    /// Hover text of the button that copies the colour's values.
    pub copy_values: String,
    /// Hover text of the colour button, shown with `Style::explanation_tooltips`.
    pub edit_color: String,
    /// Hover text of the numeric colour space button while it shows 0-255
    /// gamma values.
    pub gamma_byte_space: String,
    /// Hover text of the numeric colour space button while it shows 0-1
    /// linear values.
    pub linear_space: String,
}

impl Translations {
    /// The words in English.
    pub fn english() -> Self {
        Self {
            color_picker: ColorPickerTranslations::english(),
        }
    }
}

impl Default for Translations {
    fn default() -> Self {
        Self::english()
    }
}

impl ColorPickerTranslations {
    /// The words in English.
    pub fn english() -> Self {
        Self {
            selected_color: "Selected color".to_owned(),
            blending: "Blending:".to_owned(),
            normal_blending: "Normal".to_owned(),
            additive_blending: "Additive".to_owned(),
            hue: "Hue".to_owned(),
            saturation: "Saturation".to_owned(),
            value: "Value".to_owned(),
            alpha: "Alpha".to_owned(),
            copy_values: "Click to copy color values".to_owned(),
            edit_color: "Click to edit color".to_owned(),
            gamma_byte_space: "Showing color values in 0-255 gamma space".to_owned(),
            linear_space: "Showing color values in 0-1 linear space".to_owned(),
        }
    }
}

impl Default for ColorPickerTranslations {
    fn default() -> Self {
        Self::english()
    }
}

/// Where a context keeps the translations stated for it.
fn translations_id() -> Id {
    Id::new("egui::translations")
}

fn english() -> Arc<Translations> {
    static ENGLISH: OnceLock<Arc<Translations>> = OnceLock::new();
    ENGLISH.get_or_init(|| Arc::new(Translations::english())).clone()
}

impl Context {
    /// The words the built-in widgets of this context draw: the ones
    /// [`Self::set_translations`] stated last, else English.
    pub fn translations(&self) -> Arc<Translations> {
        self.data(|data| data.get_temp::<Arc<Translations>>(translations_id()))
            .unwrap_or_else(english)
    }

    /// State the words the built-in widgets of this context draw from now on.
    pub fn set_translations(&self, translations: impl Into<Arc<Translations>>) {
        let translations = translations.into();
        self.data_mut(|data| data.insert_temp(translations_id(), translations));
    }
}
