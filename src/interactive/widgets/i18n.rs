//! Optional localization for the interactive help pane.
//!
//! The language is selected from the standard POSIX locale environment
//! variables, honouring their conventional precedence
//! `LC_ALL` > `LC_MESSAGES` > `LANG`. English is the default and is used
//! whenever no supported language is detected, or when the supported language
//! explicitly requests a non-UTF-8 codeset. Only the help pane is translated;
//! no extra dependencies are involved.

/// A language the help pane can be rendered in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Japanese,
}

impl Language {
    /// Detect the language from the process environment.
    pub fn from_env() -> Self {
        detect([
            std::env::var("LC_ALL").ok(),
            std::env::var("LC_MESSAGES").ok(),
            std::env::var("LANG").ok(),
        ])
    }

    /// The translated strings for this language.
    pub fn help_text(self) -> &'static HelpText {
        match self {
            Language::English => &EN,
            Language::Japanese => &JA,
        }
    }
}

/// Resolve a [`Language`] from POSIX locale values, most-significant first.
///
/// Empty values are treated as unset (as glibc does) and fall through to the
/// next variable. The language code is the part before the first `_`, `.` or
/// `@`. Missing codesets are treated as UTF-8, so `ja`, `ja_JP`,
/// `ja_JP.UTF-8`, `ja.UTF8` and `ja_JP.UTF-8@modifier` map to
/// [`Language::Japanese`]. Explicit non-UTF-8 Japanese codesets such as
/// `ja_JP.SJIS` map to [`Language::English`].
fn detect<S>(locales: impl IntoIterator<Item = Option<S>>) -> Language
where
    S: AsRef<str>,
{
    let locale = locales
        .into_iter()
        .flatten()
        .find(|value| !value.as_ref().is_empty());
    match locale {
        Some(locale) if is_japanese_utf8(locale.as_ref()) => Language::Japanese,
        Some(_) => Language::English,
        None => Language::English,
    }
}

fn is_japanese_utf8(locale: &str) -> bool {
    let locale = locale.split_once('@').map_or(locale, |(locale, _)| locale);
    let (language_region, codeset) = match locale.split_once('.') {
        Some((language_region, codeset)) => (language_region, Some(codeset)),
        None => (locale, None),
    };
    let language = language_region
        .split_once('_')
        .map_or(language_region, |(language, _)| language);
    language == "ja" && codeset.is_none_or(is_utf8_codeset)
}

fn is_utf8_codeset(codeset: &str) -> bool {
    codeset.eq_ignore_ascii_case("utf-8") || codeset.eq_ignore_ascii_case("utf8")
}

/// Every translatable string of the help pane, in render order.
///
/// Key names (the green left column), the `^` continuation markers, the
/// symbolic legend and all width math stay untranslated, so they are not part
/// of this table. Adding a language means adding one more `const` below.
pub struct HelpText {
    pub block_title: &'static str,

    pub pane_control_title: &'static str,
    pub pane_q_quit: &'static str,
    pub pane_esc_close: &'static str,
    pub pane_esc_close_2: &'static str,
    pub pane_qesc_close: &'static str,
    pub pane_qesc_close_2: &'static str,
    pub pane_tab: &'static str,
    pub pane_tab_2: &'static str,
    pub pane_help_toggle: &'static str,

    pub nav_title: &'static str,
    pub nav_down: &'static str,
    pub nav_up: &'static str,
    pub nav_descend: &'static str,
    pub nav_ascend: &'static str,
    pub nav_down10: &'static str,
    pub nav_up10: &'static str,
    pub nav_top: &'static str,
    pub nav_bottom: &'static str,

    pub disp_title: &'static str,
    pub disp_sort_size: &'static str,
    pub disp_sort_mtime: &'static str,
    pub disp_show_mtime: &'static str,
    pub disp_show_mtime_2: &'static str,
    pub disp_sort_count: &'static str,
    pub disp_show_count: &'static str,
    pub disp_sort_name: &'static str,
    pub disp_cycle_bar: &'static str,

    pub oms_title: &'static str,
    pub oms_open: &'static str,
    pub oms_toggle_down: &'static str,
    pub oms_mark_down: &'static str,
    pub oms_toggle: &'static str,
    pub oms_mark_cleanup: &'static str,
    pub oms_toggle_cleanup: &'static str,
    pub oms_mark_gitignored: &'static str,
    pub oms_toggle_gitignored: &'static str,
    pub oms_toggle_all: &'static str,
    pub oms_search: &'static str,
    pub oms_search_2: &'static str,
    pub oms_refresh_one: &'static str,
    pub oms_refresh_all: &'static str,

    pub mark_title: &'static str,
    pub mark_remove: &'static str,
    pub mark_remove_all: &'static str,
    pub mark_delete: &'static str,
    pub mark_delete_2: &'static str,
    #[cfg(feature = "trash-move")]
    pub mark_trash: &'static str,
    #[cfg(feature = "trash-move")]
    pub mark_trash_2: &'static str,

    pub app_title: &'static str,
    pub app_quit: &'static str,
}

const EN: HelpText = HelpText {
    block_title: "Help",

    pane_control_title: "Pane control",
    pane_q_quit: "Close the current pane. In main view, quit (may require confirmation).",
    pane_esc_close: "Close the current pane.",
    pane_esc_close_2: "In main view, ascend to the parent directory.",
    pane_qesc_close: "Close the current pane.",
    pane_qesc_close_2: "Closes the program if no pane is open.",
    pane_tab: "Cycle between all open panes.",
    pane_tab_2: "Activate 'Marked Items' pane to delete selected files.",
    pane_help_toggle: "Show or hide this help pane.",

    nav_title: "Navigation",
    nav_down: "Move down 1 entry.",
    nav_up: "Move up 1 entry.",
    nav_descend: "Descent into the selected directory.",
    nav_ascend: "Ascent one level into the parent directory.",
    nav_down10: "Move down 10 entries.",
    nav_up10: "Move up 10 entries.",
    nav_top: "Move to the top of the list.",
    nav_bottom: "Move to the bottom of the list.",

    disp_title: "Display",
    disp_sort_size: "Toggle sort by size descending/ascending.",
    disp_sort_mtime: "Toggle sort by modified time descending/ascending.",
    disp_show_mtime: "Show modified time or cycle mtime sort mode.",
    disp_show_mtime_2: "While sorting by mtime: entry, deep newest, deep oldest.",
    disp_sort_count: "Toggle sort by entries descending/ascending.",
    disp_show_count: "Show/hide entry count.",
    disp_sort_name: "Toggle sort by name ascending/descending.",
    disp_cycle_bar: "Cycle through percentage display and bar options.",

    oms_title: "Open/Mark/Search",
    oms_open: "Open the selected entry with the associated program.",
    oms_toggle_down: "Toggle the currently selected entry and move down.",
    oms_mark_down: "Mark the currently selected entry for deletion and move down.",
    oms_toggle: "Toggle the currently selected entry.",
    oms_mark_cleanup: "Mark cleanup candidates in the current view.",
    oms_toggle_cleanup: "Toggle cleanup-candidate detection.",
    oms_mark_gitignored: "Mark Git-ignored entries in the current view.",
    oms_toggle_gitignored: "Toggle Git-ignored entry detection.",
    oms_toggle_all: "Toggle all entries.",
    oms_search: "Git-style glob search. Toggle case with 'I'.",
    oms_search_2: "Search starts from the current directory.",
    oms_refresh_one: "Refresh only the selected entry.",
    oms_refresh_all: "Refresh all entries in the current view.",

    mark_title: "Mark entries pane",
    mark_remove: "Remove the selected entry from the list.",
    mark_remove_all: "Remove all entries from the list.",
    mark_delete: "Permanently delete all marked entries without prompt.",
    mark_delete_2: "This operation cannot be undone!",
    #[cfg(feature = "trash-move")]
    mark_trash: "Move all marked entries to the trash bin.",
    #[cfg(feature = "trash-move")]
    mark_trash_2: "The entries can be restored from the trash bin.",

    app_title: "Application control",
    app_quit: "Close the application. No questions asked!",
};

const JA: HelpText = HelpText {
    block_title: "ヘルプ",

    pane_control_title: "ペイン操作",
    pane_q_quit: "現在のペインを閉じる。メイン画面では終了する（確認が必要な場合あり）。",
    pane_esc_close: "現在のペインを閉じる。",
    pane_esc_close_2: "メイン画面では親ディレクトリへ移動する。",
    pane_qesc_close: "現在のペインを閉じる。",
    pane_qesc_close_2: "開いているペインがなければプログラムを終了する。",
    pane_tab: "開いているペインを順番に切り替える。",
    pane_tab_2: "「マーク済み」ペインを有効化して選択ファイルを削除する。",
    pane_help_toggle: "このヘルプペインの表示/非表示を切り替える。",

    nav_title: "ナビゲーション",
    nav_down: "1 件下へ移動する。",
    nav_up: "1 件上へ移動する。",
    nav_descend: "選択中のディレクトリへ入る。",
    nav_ascend: "親ディレクトリへ 1 階層戻る。",
    nav_down10: "10 件下へ移動する。",
    nav_up10: "10 件上へ移動する。",
    nav_top: "リストの先頭へ移動する。",
    nav_bottom: "リストの末尾へ移動する。",

    disp_title: "表示",
    disp_sort_size: "サイズ順（降順/昇順）の並べ替えを切り替える。",
    disp_sort_mtime: "更新日時順（降順/昇順）の並べ替えを切り替える。",
    disp_show_mtime: "更新日時を表示するか、mtime の並べ替えモードを切り替える。",
    disp_show_mtime_2: "mtime で並べ替え中: エントリ、子孫の最新、子孫の最古。",
    disp_sort_count: "エントリ数順（降順/昇順）の並べ替えを切り替える。",
    disp_show_count: "エントリ数の表示/非表示を切り替える。",
    disp_sort_name: "名前順（昇順/降順）の並べ替えを切り替える。",
    disp_cycle_bar: "割合表示とバー表示の形式を順に切り替える。",

    oms_title: "開く / マーク / 検索",
    oms_open: "選択中のエントリを関連付けられたプログラムで開く。",
    oms_toggle_down: "選択中のエントリを切り替えて下へ移動する。",
    oms_mark_down: "選択中のエントリを削除対象にマークして下へ移動する。",
    oms_toggle: "選択中のエントリを切り替える。",
    oms_mark_cleanup: "現在のビューのクリーンアップ候補をマークする。",
    oms_toggle_cleanup: "クリーンアップ候補の検出を切り替える。",
    oms_mark_gitignored: "現在のビューの Git 無視エントリをマークする。",
    oms_toggle_gitignored: "Git 無視エントリの検出を切り替える。",
    oms_toggle_all: "すべてのエントリを切り替える。",
    oms_search: "Git 形式の glob 検索。'I' で大文字小文字を切り替える。",
    oms_search_2: "検索は現在のディレクトリから始まる。",
    oms_refresh_one: "選択中のエントリのみ再読み込みする。",
    oms_refresh_all: "現在のビューのすべてのエントリを再読み込みする。",

    mark_title: "マーク済みペイン",
    mark_remove: "選択中のエントリをリストから外す。",
    mark_remove_all: "すべてのエントリをリストから外す。",
    mark_delete: "マークしたすべてのエントリを確認なしで完全に削除する。",
    mark_delete_2: "この操作は取り消せません！",
    #[cfg(feature = "trash-move")]
    mark_trash: "マークしたすべてのエントリをゴミ箱へ移動する。",
    #[cfg(feature = "trash-move")]
    mark_trash_2: "エントリはゴミ箱から復元できる。",

    app_title: "アプリ操作",
    app_quit: "アプリケーションを終了する。確認なし！",
};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn defaults_to_english_when_unset() {
        assert_eq!(detect([None::<&str>, None, None]), Language::English);
    }

    #[test]
    fn japanese_locale_selects_japanese_when_codeset_is_missing_or_utf8() {
        assert_eq!(
            detect([None, None, Some("ja_JP.UTF-8")]),
            Language::Japanese
        );
        assert_eq!(detect([None, None, Some("ja")]), Language::Japanese);
        assert_eq!(detect([None, None, Some("ja_JP")]), Language::Japanese);
        assert_eq!(
            detect([None, None, Some("ja@modifier")]),
            Language::Japanese
        );
        assert_eq!(detect([None, None, Some("ja_JP.utf8")]), Language::Japanese);
        assert_eq!(detect([None, None, Some("ja.UTF-8")]), Language::Japanese);
        assert_eq!(
            detect([None, None, Some("ja_JP.UTF-8@modifier")]),
            Language::Japanese
        );
    }

    #[test]
    fn explicit_non_utf8_japanese_locales_are_english() {
        assert_eq!(
            detect([None, None, Some("ja_JP.SJIS")]),
            Language::English,
            "there is no plan to support other charsets right now, but contributions are welcome"
        );
        assert_eq!(
            detect([None, None, Some("ja_JP.EUC-JP")]),
            Language::English
        );
    }

    #[test]
    fn non_japanese_locales_are_english() {
        assert_eq!(detect([None, None, Some("en_US.UTF-8")]), Language::English);
        assert_eq!(detect([None, None, Some("C")]), Language::English);
        assert_eq!(detect([None, None, Some("POSIX")]), Language::English);
    }

    #[test]
    fn precedence_lc_all_over_lc_messages_over_lang() {
        // LC_ALL wins over everything.
        assert_eq!(
            detect([
                Some("ja_JP.UTF-8"),
                Some("en_US.UTF-8"),
                Some("en_US.UTF-8")
            ]),
            Language::Japanese
        );
        // LC_ALL wins even when it selects English.
        assert_eq!(
            detect([Some("C"), Some("ja_JP.UTF-8"), Some("ja_JP.UTF-8")]),
            Language::English
        );
        // LC_MESSAGES wins over LANG.
        assert_eq!(
            detect([None, Some("ja_JP.UTF-8"), Some("en_US.UTF-8")]),
            Language::Japanese
        );
    }

    #[test]
    fn empty_values_are_treated_as_unset() {
        // Empty LC_ALL/LC_MESSAGES fall through to LANG.
        assert_eq!(
            detect([Some(""), Some(""), Some("ja_JP.UTF-8")]),
            Language::Japanese
        );
        // All empty falls back to the default.
        assert_eq!(detect([Some(""), Some(""), Some("")]), Language::English);
    }
}
