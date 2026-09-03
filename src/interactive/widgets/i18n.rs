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
    Korean,
    Chinese,
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
            Language::Korean => &KO,
            Language::Chinese => &ZH,
        }
    }
}

/// Resolve a [`Language`] from POSIX locale values, most-significant first.
///
/// Empty values are treated as unset (as glibc does) and fall through to the
/// next variable. The language code is the part before the first `_`, `.` or
/// `@`. Missing codesets are treated as UTF-8. Supported languages with an
/// explicit non-UTF-8 codeset map to [`Language::English`].
fn detect<S>(locales: impl IntoIterator<Item = Option<S>>) -> Language
where
    S: AsRef<str>,
{
    let locale = locales
        .into_iter()
        .flatten()
        .find(|value| !value.as_ref().is_empty());
    match locale
        .as_ref()
        .and_then(|locale| utf8_language(locale.as_ref()))
    {
        Some("ja") => Language::Japanese,
        Some("ko") => Language::Korean,
        Some("zh") => Language::Chinese,
        _ => Language::English,
    }
}

fn utf8_language(locale: &str) -> Option<&str> {
    let locale = locale.split_once('@').map_or(locale, |(locale, _)| locale);
    let (language_region, codeset) = match locale.split_once('.') {
        Some((language_region, codeset)) => (language_region, Some(codeset)),
        None => (locale, None),
    };
    let language = language_region
        .split_once('_')
        .map_or(language_region, |(language, _)| language);
    codeset.is_none_or(is_utf8_codeset).then_some(language)
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
    pub app_suspend: &'static str,
    pub app_repaint: &'static str,
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
    oms_search: "Git-style glob search.",
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
    app_suspend: "Suspend the application and return control to the shell.",
    app_repaint: "Clear and repaint the screen.",
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
    oms_search: "Git 形式の glob 検索。",
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
    app_suspend: "アプリケーションを一時停止してシェルに戻る。",
    app_repaint: "画面を消去して再描画する。",
    app_quit: "アプリケーションを終了する。確認なし！",
};

const KO: HelpText = HelpText {
    block_title: "도움말",

    pane_control_title: "패널 제어",
    pane_q_quit: "현재 패널을 닫습니다. 기본 화면에서는 종료합니다(확인이 필요할 수 있음).",
    pane_esc_close: "현재 패널을 닫습니다.",
    pane_esc_close_2: "기본 화면에서는 상위 디렉터리로 이동합니다.",
    pane_qesc_close: "현재 패널을 닫습니다.",
    pane_qesc_close_2: "열린 패널이 없으면 프로그램을 종료합니다.",
    pane_tab: "열린 모든 패널을 순환합니다.",
    pane_tab_2: "'표시된 항목' 패널을 활성화하여 선택한 파일을 삭제합니다.",
    pane_help_toggle: "이 도움말 패널을 표시하거나 숨깁니다.",

    nav_title: "탐색",
    nav_down: "한 항목 아래로 이동합니다.",
    nav_up: "한 항목 위로 이동합니다.",
    nav_descend: "선택한 디렉터리로 들어갑니다.",
    nav_ascend: "상위 디렉터리로 한 단계 이동합니다.",
    nav_down10: "10개 항목 아래로 이동합니다.",
    nav_up10: "10개 항목 위로 이동합니다.",
    nav_top: "목록의 맨 위로 이동합니다.",
    nav_bottom: "목록의 맨 아래로 이동합니다.",

    disp_title: "표시",
    disp_sort_size: "크기 기준 내림차순/오름차순 정렬을 전환합니다.",
    disp_sort_mtime: "수정 시간 기준 내림차순/오름차순 정렬을 전환합니다.",
    disp_show_mtime: "수정 시간을 표시하거나 mtime 정렬 모드를 순환합니다.",
    disp_show_mtime_2: "mtime 정렬 중: 항목, 하위 항목 중 최신, 하위 항목 중 가장 오래됨.",
    disp_sort_count: "항목 수 기준 내림차순/오름차순 정렬을 전환합니다.",
    disp_show_count: "항목 수를 표시하거나 숨깁니다.",
    disp_sort_name: "이름 기준 오름차순/내림차순 정렬을 전환합니다.",
    disp_cycle_bar: "백분율 및 막대 표시 옵션을 순환합니다.",

    oms_title: "열기/표시/검색",
    oms_open: "선택한 항목을 연결된 프로그램으로 엽니다.",
    oms_toggle_down: "현재 선택한 항목의 표시 상태를 전환하고 아래로 이동합니다.",
    oms_mark_down: "현재 선택한 항목을 삭제 대상으로 표시하고 아래로 이동합니다.",
    oms_toggle: "현재 선택한 항목의 표시 상태를 전환합니다.",
    oms_mark_cleanup: "현재 보기에서 정리 후보를 표시합니다.",
    oms_toggle_cleanup: "정리 후보 감지를 전환합니다.",
    oms_mark_gitignored: "현재 보기에서 Git이 무시하는 항목을 표시합니다.",
    oms_toggle_gitignored: "Git 무시 항목 감지를 전환합니다.",
    oms_toggle_all: "모든 항목의 표시 상태를 전환합니다.",
    oms_search: "Git 방식의 glob 검색.",
    oms_search_2: "검색은 현재 디렉터리에서 시작됩니다.",
    oms_refresh_one: "선택한 항목만 새로 고칩니다.",
    oms_refresh_all: "현재 보기의 모든 항목을 새로 고칩니다.",

    mark_title: "표시된 항목 패널",
    mark_remove: "선택한 항목을 목록에서 제거합니다.",
    mark_remove_all: "모든 항목을 목록에서 제거합니다.",
    mark_delete: "표시된 모든 항목을 확인 없이 영구적으로 삭제합니다.",
    mark_delete_2: "이 작업은 취소할 수 없습니다!",
    #[cfg(feature = "trash-move")]
    mark_trash: "표시된 모든 항목을 휴지통으로 이동합니다.",
    #[cfg(feature = "trash-move")]
    mark_trash_2: "항목을 휴지통에서 복원할 수 있습니다.",

    app_title: "애플리케이션 제어",
    app_suspend: "애플리케이션을 일시 중단하고 셸로 제어권을 돌려줍니다.",
    app_repaint: "화면을 지우고 다시 그립니다.",
    app_quit: "애플리케이션을 종료합니다. 확인하지 않습니다!",
};

const ZH: HelpText = HelpText {
    block_title: "帮助",

    pane_control_title: "面板控制",
    pane_q_quit: "关闭当前面板。在主视图中退出（可能需要确认）。",
    pane_esc_close: "关闭当前面板。",
    pane_esc_close_2: "在主视图中，返回上级目录。",
    pane_qesc_close: "关闭当前面板。",
    pane_qesc_close_2: "如果没有打开的面板，则退出程序。",
    pane_tab: "在所有打开的面板之间循环切换。",
    pane_tab_2: "激活“已标记项目”面板以删除所选文件。",
    pane_help_toggle: "显示或隐藏此帮助面板。",

    nav_title: "导航",
    nav_down: "向下移动 1 个条目。",
    nav_up: "向上移动 1 个条目。",
    nav_descend: "进入所选目录。",
    nav_ascend: "返回上一级目录。",
    nav_down10: "向下移动 10 个条目。",
    nav_up10: "向上移动 10 个条目。",
    nav_top: "移到列表顶部。",
    nav_bottom: "移到列表底部。",

    disp_title: "显示",
    disp_sort_size: "在按大小降序/升序排序之间切换。",
    disp_sort_mtime: "在按修改时间降序/升序排序之间切换。",
    disp_show_mtime: "显示修改时间或循环切换 mtime 排序模式。",
    disp_show_mtime_2: "按 mtime 排序时：当前条目、子项中最新、子项中最旧。",
    disp_sort_count: "在按条目数降序/升序排序之间切换。",
    disp_show_count: "显示或隐藏条目数。",
    disp_sort_name: "在按名称升序/降序排序之间切换。",
    disp_cycle_bar: "循环切换百分比和条形图显示选项。",

    oms_title: "打开/标记/搜索",
    oms_open: "使用关联的程序打开所选条目。",
    oms_toggle_down: "切换当前所选条目的标记状态并下移。",
    oms_mark_down: "将当前所选条目标记为待删除并下移。",
    oms_toggle: "切换当前所选条目的标记状态。",
    oms_mark_cleanup: "标记当前视图中的清理候选项。",
    oms_toggle_cleanup: "切换清理候选项检测。",
    oms_mark_gitignored: "标记当前视图中被 Git 忽略的条目。",
    oms_toggle_gitignored: "切换 Git 忽略条目检测。",
    oms_toggle_all: "切换所有条目的标记状态。",
    oms_search: "Git 风格的 glob 搜索。",
    oms_search_2: "从当前目录开始搜索。",
    oms_refresh_one: "仅刷新所选条目。",
    oms_refresh_all: "刷新当前视图中的所有条目。",

    mark_title: "已标记条目面板",
    mark_remove: "从列表中移除所选条目。",
    mark_remove_all: "从列表中移除所有条目。",
    mark_delete: "不经提示永久删除所有已标记条目。",
    mark_delete_2: "此操作无法撤销！",
    #[cfg(feature = "trash-move")]
    mark_trash: "将所有已标记条目移到回收站。",
    #[cfg(feature = "trash-move")]
    mark_trash_2: "可以从回收站恢复这些条目。",

    app_title: "应用控制",
    app_suspend: "暂停应用程序并将控制权交还给 shell。",
    app_repaint: "清空并重绘屏幕。",
    app_quit: "直接关闭应用程序，不作确认！",
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
    fn korean_locale_selects_korean_when_codeset_is_missing_or_utf8() {
        assert_eq!(detect([None, None, Some("ko_KR.UTF-8")]), Language::Korean);
        assert_eq!(detect([None, None, Some("ko")]), Language::Korean);
    }

    #[test]
    fn chinese_locale_selects_chinese_when_codeset_is_missing_or_utf8() {
        assert_eq!(detect([None, None, Some("zh_CN.UTF-8")]), Language::Chinese);
        assert_eq!(detect([None, None, Some("zh")]), Language::Chinese);
    }

    #[test]
    fn explicit_non_utf8_supported_locales_are_english() {
        assert_eq!(
            detect([None, None, Some("ja_JP.SJIS")]),
            Language::English,
            "there is no plan to support other charsets right now, but contributions are welcome"
        );
        assert_eq!(
            detect([None, None, Some("ja_JP.EUC-JP")]),
            Language::English
        );
        assert_eq!(
            detect([None, None, Some("ko_KR.EUC-KR")]),
            Language::English
        );
        assert_eq!(
            detect([None, None, Some("zh_CN.GB18030")]),
            Language::English
        );
    }

    #[test]
    fn unsupported_locales_are_english() {
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
