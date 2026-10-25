/// The words the date picker draws: its month and weekday names, its
/// buttons and the hover texts of its arrows.
///
/// [`DatePickerButton::translations`](crate::DatePickerButton::translations)
/// states them for one picker; a picker given none draws [`Self::english`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DatePickerTranslations {
    /// The months, January first.
    pub months: [String; 12],
    /// The weekday column headers, Monday first.
    pub weekdays: [String; 7],
    /// Header of the calendar week column.
    pub week: String,
    /// Button that closes the picker without changing the date.
    pub cancel: String,
    /// Button that takes the picked date.
    pub save: String,
    /// Hover text of the arrow that moves back one year.
    pub subtract_year: String,
    /// Hover text of the arrow that moves back one month.
    pub subtract_month: String,
    /// Hover text of the arrow that moves back one day.
    pub subtract_day: String,
    /// Hover text of the arrow that moves forward one day.
    pub add_day: String,
    /// Hover text of the arrow that moves forward one month.
    pub add_month: String,
    /// Hover text of the arrow that moves forward one year.
    pub add_year: String,
}

impl DatePickerTranslations {
    /// The words in English.
    pub fn english() -> Self {
        Self {
            months: [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]
            .map(str::to_owned),
            weekdays: ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"].map(str::to_owned),
            week: "Week".to_owned(),
            cancel: "Cancel".to_owned(),
            save: "Save".to_owned(),
            subtract_year: "subtract one year".to_owned(),
            subtract_month: "subtract one month".to_owned(),
            subtract_day: "subtract one day".to_owned(),
            add_day: "add one day".to_owned(),
            add_month: "add one month".to_owned(),
            add_year: "add one year".to_owned(),
        }
    }

    /// The name of `month`, 1 for January through 12 for December.
    pub fn month(&self, month: i8) -> &str {
        let index = usize::try_from(month - 1).unwrap_or(usize::MAX);
        self.months.get(index).map_or("", String::as_str)
    }

    /// The English words, shared by every picker given none.
    pub(crate) fn shared_english() -> &'static Self {
        static ENGLISH: std::sync::OnceLock<DatePickerTranslations> = std::sync::OnceLock::new();
        ENGLISH.get_or_init(Self::english)
    }
}

impl Default for DatePickerTranslations {
    fn default() -> Self {
        Self::english()
    }
}
