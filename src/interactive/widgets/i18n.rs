//! Optional localization for the interactive interface.
//!
//! The language is selected from the standard POSIX locale environment
//! variables, honouring their conventional precedence
//! `LC_ALL` > `LC_MESSAGES` > `LANG`. English is the default and is used
//! whenever no supported language is detected, or when the supported language
//! explicitly requests a non-UTF-8 codeset. No extra dependencies are involved.

/// A language the interactive interface can be rendered in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Japanese,
    Korean,
    Chinese,
    German,
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
            Language::German => &DE,
        }
    }

    /// The translated strings used by the rest of the interactive interface.
    pub fn ui_text(self) -> &'static UiText {
        match self {
            Language::English => &EN_UI,
            Language::Japanese => &JA_UI,
            Language::Korean => &KO_UI,
            Language::Chinese => &ZH_UI,
            Language::German => &DE_UI,
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
        .and_then(|locale| utf8_locale(locale.as_ref()))
    {
        Some(("de", _)) => Language::German,
        Some(("ja", _)) => Language::Japanese,
        Some(("ko", _)) => Language::Korean,
        Some(("zh", "zh" | "zh_CN" | "zh_SG" | "zh_Hans")) => Language::Chinese,
        _ => Language::English,
    }
}

fn utf8_locale(locale: &str) -> Option<(&str, &str)> {
    let locale = locale.split_once('@').map_or(locale, |(locale, _)| locale);
    let (language_region, codeset) = match locale.split_once('.') {
        Some((language_region, codeset)) => (language_region, Some(codeset)),
        None => (locale, None),
    };
    let language = language_region
        .split_once('_')
        .map_or(language_region, |(language, _)| language);
    codeset
        .is_none_or(is_utf8_codeset)
        .then_some((language, language_region))
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

const DE: HelpText = HelpText {
    block_title: "Hilfe",

    pane_control_title: "Bereichssteuerung",
    pane_q_quit: "Bereich schließen; Hauptansicht beenden (evtl. bestätigen).",
    pane_esc_close: "Aktuellen Bereich schließen.",
    pane_esc_close_2: "In der Hauptansicht zum übergeordneten Verzeichnis wechseln.",
    pane_qesc_close: "Aktuellen Bereich schließen.",
    pane_qesc_close_2: "Beendet das Programm, wenn kein Bereich geöffnet ist.",
    pane_tab: "Zwischen allen geöffneten Bereichen wechseln.",
    pane_tab_2: "„Markierte Einträge“ zum Löschen gewählter Dateien öffnen.",
    pane_help_toggle: "Diesen Hilfebereich ein- oder ausblenden.",

    nav_title: "Navigation",
    nav_down: "Einen Eintrag nach unten bewegen.",
    nav_up: "Einen Eintrag nach oben bewegen.",
    nav_descend: "In das ausgewählte Verzeichnis wechseln.",
    nav_ascend: "Eine Ebene ins übergeordnete Verzeichnis wechseln.",
    nav_down10: "10 Einträge nach unten bewegen.",
    nav_up10: "10 Einträge nach oben bewegen.",
    nav_top: "Zum Anfang der Liste springen.",
    nav_bottom: "Zum Ende der Liste springen.",

    disp_title: "Anzeige",
    disp_sort_size: "Sortierung nach Größe absteigend/aufsteigend umschalten.",
    disp_sort_mtime: "Nach Änderungszeit ab-/aufsteigend sortieren.",
    disp_show_mtime: "Änderungszeit anzeigen oder mtime-Sortiermodus wechseln.",
    disp_show_mtime_2: "mtime-Sortierung: Eintrag, neuester/ältester Untereintrag.",
    disp_sort_count: "Nach Eintragsanzahl ab-/aufsteigend sortieren.",
    disp_show_count: "Eintragsanzahl ein- oder ausblenden.",
    disp_sort_name: "Nach Namen auf-/absteigend sortieren.",
    disp_cycle_bar: "Zwischen Prozent- und Balkenanzeige wechseln.",

    oms_title: "Öffnen/Markieren/Suchen",
    oms_open: "Ausgewählten Eintrag mit dem zugeordneten Programm öffnen.",
    oms_toggle_down: "Markierung umschalten und einen Eintrag nach unten gehen.",
    oms_mark_down: "Zum Löschen markieren und einen Eintrag nach unten gehen.",
    oms_toggle: "Markierung des ausgewählten Eintrags umschalten.",
    oms_mark_cleanup: "Bereinigungskandidaten in der aktuellen Ansicht markieren.",
    oms_toggle_cleanup: "Erkennung von Bereinigungskandidaten umschalten.",
    oms_mark_gitignored: "Von Git ignorierte Einträge dieser Ansicht markieren.",
    oms_toggle_gitignored: "Erkennung von Git-ignorierten Einträgen umschalten.",
    oms_toggle_all: "Markierung aller Einträge umschalten.",
    oms_search: "Glob-Suche im Git-Stil.",
    oms_search_2: "Die Suche beginnt im aktuellen Verzeichnis.",
    oms_refresh_one: "Nur den ausgewählten Eintrag aktualisieren.",
    oms_refresh_all: "Alle Einträge in der aktuellen Ansicht aktualisieren.",

    mark_title: "Bereich „Markierte Einträge“",
    mark_remove: "Ausgewählten Eintrag aus der Liste entfernen.",
    mark_remove_all: "Alle Einträge aus der Liste entfernen.",
    mark_delete: "Alle markierten Einträge ohne Rückfrage endgültig löschen.",
    mark_delete_2: "Dieser Vorgang kann nicht rückgängig gemacht werden!",
    #[cfg(feature = "trash-move")]
    mark_trash: "Alle markierten Einträge in den Papierkorb verschieben.",
    #[cfg(feature = "trash-move")]
    mark_trash_2: "Einträge können aus dem Papierkorb wiederhergestellt werden.",

    app_title: "Anwendungssteuerung",
    app_suspend: "Anwendung anhalten und Steuerung an die Shell zurückgeben.",
    app_repaint: "Bildschirm leeren und neu zeichnen.",
    app_quit: "Anwendung ohne Rückfrage schließen!",
};

/// Static text used outside the help pane.
pub struct UiText {
    pub header_help_before_key: &'static str,
    pub header_help_after_key: &'static str,

    pub entries_mark_move: &'static str,
    pub entries_mark_toggle: &'static str,
    pub entries_cleanup: &'static str,
    pub entries_gitignore: &'static str,
    pub entries_all: &'static str,

    pub glob_case_sensitive: &'static str,
    pub glob_case_insensitive: &'static str,
    pub glob_search: &'static str,
    pub glob_case: &'static str,
    pub glob_cancel: &'static str,
    pub glob_empty: &'static str,

    pub mark_snapshot_read_only: &'static str,
    pub mark_no_destructive_keys: &'static str,
    #[cfg(feature = "trash-move")]
    pub mark_to_trash_or: &'static str,
    pub mark_to_delete: &'static str,
    pub mark_toggle: &'static str,
    pub mark_remove_all: &'static str,

    pub footer_sort_mode: &'static str,
    pub footer_total_disk_usage: &'static str,
    pub sort_size_ascending: &'static str,
    pub sort_size_descending: &'static str,
    pub sort_mtime_ascending: &'static str,
    pub sort_mtime_descending: &'static str,
    pub sort_count_ascending: &'static str,
    pub sort_count_descending: &'static str,
    pub sort_name_ascending: &'static str,
    pub sort_name_descending: &'static str,
    pub sort_deep_newest: &'static str,
    pub sort_deep_oldest: &'static str,

    pub snapshot_label: &'static str,
    pub snapshot_path_unavailable: &'static str,
    pub snapshot_temporary_failed: &'static str,
    pub snapshot_write_failed: &'static str,
    pub snapshot_install_failed: &'static str,
    pub failed_to_open: &'static str,
    pub top_level: &'static str,
    pub entry_file_or_empty: &'static str,
    pub gitignore_snapshot_unavailable: &'static str,
    pub scanning: &'static str,
    pub snapshots_read_only: &'static str,
    pub traversal_running: &'static str,
    pub deleting_items: &'static str,
    #[cfg(feature = "trash-move")]
    pub trashing_items: &'static str,
    pub no_cleanup_candidates: &'static str,
    pub cleanup_candidates_already_marked: &'static str,
    pub cleanup_detection_disabled: &'static str,
    pub no_gitignored_entries: &'static str,
    pub gitignored_entries_already_marked: &'static str,
    pub gitignore_detection_disabled: &'static str,
    pub no_match: &'static str,

    pub notification_scan: &'static str,
    pub notification_deletion: &'static str,
    #[cfg(feature = "trash-move")]
    pub notification_trash: &'static str,
    pub notification_finished: &'static str,
}

impl Language {
    pub fn entries_statistics(self, visible: usize, total: &str, size: &str) -> String {
        match self {
            Language::English => format!("({visible} visible, {total} total, {size})"),
            Language::Japanese => format!("(表示 {visible} 件、合計 {total} 件、{size})"),
            Language::Korean => format!("(표시 {visible}개, 전체 {total}개, {size})"),
            Language::Chinese => format!("(显示 {visible} 项，共 {total} 项，{size})"),
            Language::German => format!("({visible} sichtbar, {total} gesamt, {size})"),
        }
    }

    pub fn marked_title(self, count: &str, size: &str, percentage: f64, root_size: &str) -> String {
        match self {
            Language::English => {
                format!("Marked {count} items ({size}, {percentage:.2}% of {root_size}) ")
            }
            Language::Japanese => {
                format!("マーク済み {count} 件 ({size}、{root_size} の {percentage:.2}%) ")
            }
            Language::Korean => {
                format!("표시된 항목 {count}개 ({size}, {root_size}의 {percentage:.2}%) ")
            }
            Language::Chinese => {
                format!("已标记 {count} 个条目（{size}，占 {root_size} 的 {percentage:.2}%） ")
            }
            Language::German => {
                format!("{count} Einträge markiert ({size}, {percentage:.2}% von {root_size}) ")
            }
        }
    }

    pub fn deletion_errors(self, count: usize) -> String {
        match self {
            Language::English => format!("{count} IO deletion errors"),
            Language::Japanese => format!("削除 I/O エラー {count} 件"),
            Language::Korean => format!("I/O 삭제 오류 {count}건"),
            Language::Chinese => format!("{count} 个 I/O 删除错误"),
            Language::German => format!("{count} E/A-Löschfehler"),
        }
    }

    pub fn exit_message(self, quit: &str, close: Option<&str>) -> String {
        match (self, close) {
            (Language::English, None) => format!("Press {quit} again to exit..."),
            (Language::English, Some(close)) => {
                format!("Press {close} or {quit} again to exit...")
            }
            (Language::Japanese, None) => format!("{quit} をもう一度押すと終了します..."),
            (Language::Japanese, Some(close)) => {
                format!("{close} または {quit} をもう一度押すと終了します...")
            }
            (Language::Korean, None) => format!("종료하려면 {quit} 키를 다시 누르세요..."),
            (Language::Korean, Some(close)) => {
                format!("종료하려면 {close} 또는 {quit} 키를 다시 누르세요...")
            }
            (Language::Chinese, None) => format!("再次按 {quit} 退出..."),
            (Language::Chinese, Some(close)) => {
                format!("再次按 {close} 或 {quit} 退出...")
            }
            (Language::German, None) => format!("Zum Beenden {quit} erneut drücken..."),
            (Language::German, Some(close)) => {
                format!("Zum Beenden {close} oder {quit} erneut drücken...")
            }
        }
    }

    pub fn footer_progress(self, entries: u64, progress: &str) -> String {
        match self {
            Language::English => format!("Processed {entries} entries in {progress}  "),
            Language::Japanese => format!("{entries} 件を {progress} で処理  "),
            Language::Korean => format!("{entries}개 항목을 {progress} 동안 처리  "),
            Language::Chinese => format!("已处理 {entries} 个条目，用时 {progress}  "),
            Language::German => {
                let label = if entries == 1 { "Eintrag" } else { "Einträge" };
                format!("{entries} {label} in {progress} verarbeitet  ")
            }
        }
    }

    pub fn top_level_with_scan(self, key: &str) -> String {
        match self {
            Language::English => {
                format!("Top level reached. Press {key} to scan the parent directory")
            }
            Language::Japanese => {
                format!("最上位に到達しました。{key} で親ディレクトリをスキャンします")
            }
            Language::Korean => {
                format!("최상위에 도달했습니다. {key} 키로 상위 디렉터리를 스캔하세요")
            }
            Language::Chinese => format!("已到达顶层。按 {key} 扫描上级目录"),
            Language::German => {
                format!("Oberste Ebene erreicht. Mit {key} das übergeordnete Verzeichnis scannen")
            }
        }
    }

    pub fn deletion_progress(self, count: usize, trash: bool) -> String {
        match (self, trash) {
            (Language::English, false) => format!("Deleted {count} items..."),
            (Language::English, true) => format!("Trashed {count} items..."),
            (Language::Japanese, false) => format!("{count} 件を削除..."),
            (Language::Japanese, true) => format!("{count} 件をゴミ箱へ移動..."),
            (Language::Korean, false) => format!("{count}개 항목 삭제..."),
            (Language::Korean, true) => format!("{count}개 항목을 휴지통으로 이동..."),
            (Language::Chinese, false) => format!("已删除 {count} 个条目..."),
            (Language::Chinese, true) => format!("已将 {count} 个条目移至回收站..."),
            (Language::German, false) => format!(
                "{count} {} gelöscht...",
                if count == 1 { "Eintrag" } else { "Einträge" }
            ),
            (Language::German, true) => {
                let label = if count == 1 { "Eintrag" } else { "Einträge" };
                format!("{count} {label} in den Papierkorb verschoben...")
            }
        }
    }

    pub fn marked_candidates(self, count: usize, gitignored: bool) -> String {
        match (self, gitignored) {
            (Language::English, false) => format!("Marked {count} cleanup candidates"),
            (Language::English, true) => format!("Marked {count} gitignored entries"),
            (Language::Japanese, false) => {
                format!("クリーンアップ候補を {count} 件マーク")
            }
            (Language::Japanese, true) => format!("Git 無視エントリを {count} 件マーク"),
            (Language::Korean, false) => format!("정리 후보 {count}개 표시"),
            (Language::Korean, true) => format!("Git 무시 항목 {count}개 표시"),
            (Language::Chinese, false) => format!("已标记 {count} 个清理候选项"),
            (Language::Chinese, true) => format!("已标记 {count} 个 Git 忽略条目"),
            (Language::German, false) => {
                let label = if count == 1 {
                    "Bereinigungskandidat"
                } else {
                    "Bereinigungskandidaten"
                };
                format!("{count} {label} markiert")
            }
            (Language::German, true) => {
                let label = if count == 1 { "Eintrag" } else { "Einträge" };
                format!("{count} von Git ignorierte {label} markiert")
            }
        }
    }

    pub fn annotation_message(self, cleanup: usize, gitignored: usize) -> Option<String> {
        match (self, cleanup, gitignored) {
            (_, 0, 0) => None,
            (Language::English, cleanup, 0) => Some(format!(
                "{cleanup} cleanup candidate{}",
                if cleanup == 1 { "" } else { "s" }
            )),
            (Language::English, 0, gitignored) => Some(format!(
                "{gitignored} gitignored {}",
                if gitignored == 1 { "entry" } else { "entries" }
            )),
            (Language::English, cleanup, gitignored) => {
                Some(format!("{cleanup} cleanup, {gitignored} gitignored"))
            }
            (Language::Japanese, cleanup, 0) => Some(format!("クリーンアップ候補 {cleanup} 件")),
            (Language::Japanese, 0, gitignored) => {
                Some(format!("Git 無視エントリ {gitignored} 件"))
            }
            (Language::Japanese, cleanup, gitignored) => Some(format!(
                "クリーンアップ {cleanup} 件、Git 無視 {gitignored} 件"
            )),
            (Language::Korean, cleanup, 0) => Some(format!("정리 후보 {cleanup}개")),
            (Language::Korean, 0, gitignored) => Some(format!("Git 무시 항목 {gitignored}개")),
            (Language::Korean, cleanup, gitignored) => {
                Some(format!("정리 {cleanup}개, Git 무시 {gitignored}개"))
            }
            (Language::Chinese, cleanup, 0) => Some(format!("{cleanup} 个清理候选项")),
            (Language::Chinese, 0, gitignored) => Some(format!("{gitignored} 个 Git 忽略条目")),
            (Language::Chinese, cleanup, gitignored) => {
                Some(format!("清理 {cleanup} 个，Git 忽略 {gitignored} 个"))
            }
            (Language::German, cleanup, 0) => Some(format!(
                "{cleanup} Bereinigungskandidat{}",
                if cleanup == 1 { "" } else { "en" }
            )),
            (Language::German, 0, gitignored) => Some(format!(
                "{gitignored} von Git ignorierter Eintrag{}",
                if gitignored == 1 { "" } else { "e" }
            )),
            (Language::German, cleanup, gitignored) => {
                Some(format!("{cleanup} Bereinigung, {gitignored} Git-ignoriert"))
            }
        }
    }

    pub fn notification_summary(
        self,
        action: &str,
        entries: u64,
        bytes: &str,
        elapsed: &str,
        errors: u64,
    ) -> String {
        let errors = match (self, errors) {
            (_, 0) => String::new(),
            (Language::English, count) => format!(", {count} errors"),
            (Language::Japanese, count) => format!("、エラー {count} 件"),
            (Language::Korean, count) => format!(", 오류 {count}건"),
            (Language::Chinese, count) => format!("，{count} 个错误"),
            (Language::German, count) => format!(", {count} Fehler"),
        };
        let t = self.ui_text();
        match self {
            Language::English => {
                format!(
                    "{action} {}: {entries} entries, {bytes} in {elapsed}{errors}",
                    t.notification_finished
                )
            }
            Language::Japanese => format!(
                "{action}{}: {entries} 件、{bytes}、所要時間 {elapsed}{errors}",
                t.notification_finished
            ),
            Language::Korean => format!(
                "{action} {}: {entries}개 항목, {bytes}, 소요 시간 {elapsed}{errors}",
                t.notification_finished
            ),
            Language::Chinese => format!(
                "{action}{}：{entries} 个条目，{bytes}，用时 {elapsed}{errors}",
                t.notification_finished
            ),
            Language::German => {
                let label = if entries == 1 { "Eintrag" } else { "Einträge" };
                format!(
                    "{action} {}: {entries} {label}, {bytes} in {elapsed}{errors}",
                    t.notification_finished
                )
            }
        }
    }
}

const EN_UI: UiText = UiText {
    header_help_before_key: "(press ",
    header_help_after_key: " for help)",
    entries_mark_move: "mark-move",
    entries_mark_toggle: "mark-toggle",
    entries_cleanup: "cleanup",
    entries_gitignore: "gitignore",
    entries_all: "all",
    glob_case_sensitive: "Git-Glob (case-sensitive)",
    glob_case_insensitive: "Git-Glob (case-insensitive)",
    glob_search: "search",
    glob_case: "case",
    glob_cancel: "cancel",
    glob_empty: "Glob was empty or only whitespace",
    mark_snapshot_read_only: " Snapshot is read-only; marked entries cannot be deleted ",
    mark_no_destructive_keys: " No destructive keys are mapped; marked entries are safe ",
    #[cfg(feature = "trash-move")]
    mark_to_trash_or: " to trash or ",
    mark_to_delete: " to delete without prompt",
    mark_toggle: "mark-toggle",
    mark_remove_all: "remove-all",
    footer_sort_mode: "Sort mode",
    footer_total_disk_usage: "Total disk usage",
    sort_size_ascending: "size, small first",
    sort_size_descending: "size, large first",
    sort_mtime_ascending: "mtime, old first",
    sort_mtime_descending: "mtime, new first",
    sort_count_ascending: "items, few first",
    sort_count_descending: "items, most first",
    sort_name_ascending: "name, A-Z",
    sort_name_descending: "name, Z-A",
    sort_deep_newest: "deep newest",
    sort_deep_oldest: "deep oldest",
    snapshot_label: "<snapshot>",
    snapshot_path_unavailable: "Snapshot path is unavailable: ",
    snapshot_temporary_failed: "Could not create a temporary snapshot beside ",
    snapshot_write_failed: "Could not write snapshot to ",
    snapshot_install_failed: "Could not install snapshot at ",
    failed_to_open: "Failed to open ",
    top_level: "Top level reached",
    entry_file_or_empty: "Entry is a file or an empty directory",
    gitignore_snapshot_unavailable: "Gitignored entry detection is unavailable for snapshots",
    scanning: "-> scanning <-",
    snapshots_read_only: "Snapshots are read-only",
    traversal_running: "Traversal already running",
    deleting_items: "Deleting items...",
    #[cfg(feature = "trash-move")]
    trashing_items: "Trashing items...",
    no_cleanup_candidates: "No cleanup candidates in view",
    cleanup_candidates_already_marked: "Cleanup candidates are already marked",
    cleanup_detection_disabled: "Cleanup candidate detection is disabled",
    no_gitignored_entries: "No gitignored entries in view",
    gitignored_entries_already_marked: "Gitignored entries are already marked",
    gitignore_detection_disabled: "Gitignored entry detection is disabled",
    no_match: "No match found",
    notification_scan: "Scan",
    notification_deletion: "Deletion",
    #[cfg(feature = "trash-move")]
    notification_trash: "Trash",
    notification_finished: "finished",
};

const JA_UI: UiText = UiText {
    header_help_before_key: "(",
    header_help_after_key: "でヘルプ)",
    entries_mark_move: "マーク↓",
    entries_mark_toggle: "マーク切替",
    entries_cleanup: "整理",
    entries_gitignore: "Git無視",
    entries_all: "全て",
    glob_case_sensitive: "Git-Glob (大文字小文字を区別)",
    glob_case_insensitive: "Git-Glob (大文字小文字を区別しない)",
    glob_search: "検索",
    glob_case: "大小",
    glob_cancel: "取消",
    glob_empty: "glob が空か空白のみです",
    mark_snapshot_read_only: " スナップショットは読み取り専用のため削除できません ",
    mark_no_destructive_keys: " 削除キーは未設定です。マーク済み項目は安全です ",
    #[cfg(feature = "trash-move")]
    mark_to_trash_or: " でゴミ箱へ、または ",
    mark_to_delete: " で確認なしに削除",
    mark_toggle: "マーク切替",
    mark_remove_all: "全解除",
    footer_sort_mode: "並べ替え",
    footer_total_disk_usage: "ディスク使用量合計",
    sort_size_ascending: "サイズ、小さい順",
    sort_size_descending: "サイズ、大きい順",
    sort_mtime_ascending: "mtime、古い順",
    sort_mtime_descending: "mtime、新しい順",
    sort_count_ascending: "項目数、少ない順",
    sort_count_descending: "項目数、多い順",
    sort_name_ascending: "名前、A-Z",
    sort_name_descending: "名前、Z-A",
    sort_deep_newest: "子孫の最新",
    sort_deep_oldest: "子孫の最古",
    snapshot_label: "<スナップショット>",
    snapshot_path_unavailable: "スナップショットのパスを利用できません: ",
    snapshot_temporary_failed: "次の場所に一時スナップショットを作成できませんでした: ",
    snapshot_write_failed: "スナップショットを書き込めませんでした: ",
    snapshot_install_failed: "スナップショットを配置できませんでした: ",
    failed_to_open: "開けませんでした: ",
    top_level: "最上位に到達しました",
    entry_file_or_empty: "エントリはファイルまたは空のディレクトリです",
    gitignore_snapshot_unavailable: "スナップショットでは Git 無視エントリを検出できません",
    scanning: "-> スキャン中 <-",
    snapshots_read_only: "スナップショットは読み取り専用です",
    traversal_running: "スキャンはすでに実行中です",
    deleting_items: "項目を削除中...",
    #[cfg(feature = "trash-move")]
    trashing_items: "項目をゴミ箱へ移動中...",
    no_cleanup_candidates: "現在の表示にクリーンアップ候補はありません",
    cleanup_candidates_already_marked: "クリーンアップ候補はすでにマーク済みです",
    cleanup_detection_disabled: "クリーンアップ候補の検出は無効です",
    no_gitignored_entries: "現在の表示に Git 無視エントリはありません",
    gitignored_entries_already_marked: "Git 無視エントリはすでにマーク済みです",
    gitignore_detection_disabled: "Git 無視エントリの検出は無効です",
    no_match: "一致する項目がありません",
    notification_scan: "スキャン",
    notification_deletion: "削除",
    #[cfg(feature = "trash-move")]
    notification_trash: "ゴミ箱への移動",
    notification_finished: "完了",
};

const KO_UI: UiText = UiText {
    header_help_before_key: "(",
    header_help_after_key: "를 눌러 도움말)",
    entries_mark_move: "표시↓",
    entries_mark_toggle: "표시전환",
    entries_cleanup: "정리",
    entries_gitignore: "Git무시",
    entries_all: "전체",
    glob_case_sensitive: "Git-Glob (대소문자 구분)",
    glob_case_insensitive: "Git-Glob (대소문자 무시)",
    glob_search: "검색",
    glob_case: "대소문자",
    glob_cancel: "취소",
    glob_empty: "glob이 비어 있거나 공백뿐입니다",
    mark_snapshot_read_only: " 스냅샷은 읽기 전용이므로 표시된 항목을 삭제할 수 없습니다 ",
    mark_no_destructive_keys: " 삭제 키가 지정되지 않아 표시된 항목은 안전합니다 ",
    #[cfg(feature = "trash-move")]
    mark_to_trash_or: "로 휴지통 이동 또는 ",
    mark_to_delete: "로 확인 없이 삭제",
    mark_toggle: "표시전환",
    mark_remove_all: "모두해제",
    footer_sort_mode: "정렬",
    footer_total_disk_usage: "총 디스크 사용량",
    sort_size_ascending: "크기, 작은 순",
    sort_size_descending: "크기, 큰 순",
    sort_mtime_ascending: "mtime, 오래된 순",
    sort_mtime_descending: "mtime, 최신 순",
    sort_count_ascending: "항목 수, 적은 순",
    sort_count_descending: "항목 수, 많은 순",
    sort_name_ascending: "이름, A-Z",
    sort_name_descending: "이름, Z-A",
    sort_deep_newest: "하위 항목 중 최신",
    sort_deep_oldest: "하위 항목 중 가장 오래됨",
    snapshot_label: "<스냅샷>",
    snapshot_path_unavailable: "스냅샷 경로를 사용할 수 없습니다: ",
    snapshot_temporary_failed: "다음 위치에 임시 스냅샷을 만들 수 없습니다: ",
    snapshot_write_failed: "스냅샷을 쓸 수 없습니다: ",
    snapshot_install_failed: "스냅샷을 설치할 수 없습니다: ",
    failed_to_open: "열지 못했습니다: ",
    top_level: "최상위에 도달했습니다",
    entry_file_or_empty: "항목이 파일이거나 빈 디렉터리입니다",
    gitignore_snapshot_unavailable: "스냅샷에서는 Git 무시 항목 감지를 사용할 수 없습니다",
    scanning: "-> 스캔 중 <-",
    snapshots_read_only: "스냅샷은 읽기 전용입니다",
    traversal_running: "스캔이 이미 실행 중입니다",
    deleting_items: "항목 삭제 중...",
    #[cfg(feature = "trash-move")]
    trashing_items: "항목을 휴지통으로 이동 중...",
    no_cleanup_candidates: "현재 보기에 정리 후보가 없습니다",
    cleanup_candidates_already_marked: "정리 후보가 이미 표시되어 있습니다",
    cleanup_detection_disabled: "정리 후보 감지가 비활성화되어 있습니다",
    no_gitignored_entries: "현재 보기에 Git 무시 항목이 없습니다",
    gitignored_entries_already_marked: "Git 무시 항목이 이미 표시되어 있습니다",
    gitignore_detection_disabled: "Git 무시 항목 감지가 비활성화되어 있습니다",
    no_match: "일치하는 항목이 없습니다",
    notification_scan: "스캔",
    notification_deletion: "삭제",
    #[cfg(feature = "trash-move")]
    notification_trash: "휴지통 이동",
    notification_finished: "완료",
};

const ZH_UI: UiText = UiText {
    header_help_before_key: "(按 ",
    header_help_after_key: " 查看帮助)",
    entries_mark_move: "标记↓",
    entries_mark_toggle: "切换标记",
    entries_cleanup: "清理",
    entries_gitignore: "Git忽略",
    entries_all: "全部",
    glob_case_sensitive: "Git-Glob (区分大小写)",
    glob_case_insensitive: "Git-Glob (不区分大小写)",
    glob_search: "搜索",
    glob_case: "大小写",
    glob_cancel: "取消",
    glob_empty: "glob 为空或仅包含空白",
    mark_snapshot_read_only: " 快照为只读；无法删除已标记条目 ",
    mark_no_destructive_keys: " 未映射破坏性按键；已标记条目是安全的 ",
    #[cfg(feature = "trash-move")]
    mark_to_trash_or: " 移至回收站，或 ",
    mark_to_delete: " 无提示删除",
    mark_toggle: "切换标记",
    mark_remove_all: "全部移除",
    footer_sort_mode: "排序",
    footer_total_disk_usage: "磁盘总用量",
    sort_size_ascending: "大小，小的优先",
    sort_size_descending: "大小，大的优先",
    sort_mtime_ascending: "mtime，旧的优先",
    sort_mtime_descending: "mtime，新的优先",
    sort_count_ascending: "条目数，少的优先",
    sort_count_descending: "条目数，多的优先",
    sort_name_ascending: "名称，A-Z",
    sort_name_descending: "名称，Z-A",
    sort_deep_newest: "子项中最新",
    sort_deep_oldest: "子项中最旧",
    snapshot_label: "<快照>",
    snapshot_path_unavailable: "快照路径不可用: ",
    snapshot_temporary_failed: "无法在目标旁创建临时快照: ",
    snapshot_write_failed: "无法写入快照: ",
    snapshot_install_failed: "无法安装快照: ",
    failed_to_open: "无法打开: ",
    top_level: "已到达顶层",
    entry_file_or_empty: "条目是文件或空目录",
    gitignore_snapshot_unavailable: "快照无法使用 Git 忽略条目检测",
    scanning: "-> 正在扫描 <-",
    snapshots_read_only: "快照为只读",
    traversal_running: "扫描已在进行",
    deleting_items: "正在删除条目...",
    #[cfg(feature = "trash-move")]
    trashing_items: "正在将条目移至回收站...",
    no_cleanup_candidates: "当前视图中没有清理候选项",
    cleanup_candidates_already_marked: "清理候选项已标记",
    cleanup_detection_disabled: "清理候选项检测已禁用",
    no_gitignored_entries: "当前视图中没有 Git 忽略条目",
    gitignored_entries_already_marked: "Git 忽略条目已标记",
    gitignore_detection_disabled: "Git 忽略条目检测已禁用",
    no_match: "未找到匹配项",
    notification_scan: "扫描",
    notification_deletion: "删除",
    #[cfg(feature = "trash-move")]
    notification_trash: "移至回收站",
    notification_finished: "完成",
};

const DE_UI: UiText = UiText {
    header_help_before_key: "(",
    header_help_after_key: " für Hilfe)",
    entries_mark_move: "mark-move",
    entries_mark_toggle: "mark-toggle",
    entries_cleanup: "cleanup",
    entries_gitignore: "gitignore",
    entries_all: "all",
    glob_case_sensitive: "Git-Glob (Groß/Klein beachten)",
    glob_case_insensitive: "Git-Glob (Groß/Klein ignorieren)",
    glob_search: "Suche",
    glob_case: "case",
    glob_cancel: "cancel",
    glob_empty: "Glob ist leer oder enthält nur Leerzeichen",
    mark_snapshot_read_only: " Snapshot ist schreibgeschützt; markierte Einträge sind nicht löschbar ",
    mark_no_destructive_keys: " Keine Lösch-Tasten belegt; markierte Einträge sind sicher ",
    #[cfg(feature = "trash-move")]
    mark_to_trash_or: " in den Papierkorb oder ",
    mark_to_delete: " ohne Rückfrage löschen",
    mark_toggle: "mark-toggle",
    mark_remove_all: "remove-all",
    footer_sort_mode: "Sortierung",
    footer_total_disk_usage: "Gesamte Speicherbelegung",
    sort_size_ascending: "Größe, klein zuerst",
    sort_size_descending: "Größe, groß zuerst",
    sort_mtime_ascending: "mtime, alt zuerst",
    sort_mtime_descending: "mtime, neu zuerst",
    sort_count_ascending: "Einträge, wenige zuerst",
    sort_count_descending: "Einträge, meiste zuerst",
    sort_name_ascending: "Name, A-Z",
    sort_name_descending: "Name, Z-A",
    sort_deep_newest: "neuester Untereintrag",
    sort_deep_oldest: "ältester Untereintrag",
    snapshot_label: "<Snapshot>",
    snapshot_path_unavailable: "Snapshot-Pfad ist nicht verfügbar: ",
    snapshot_temporary_failed: "Temporärer Snapshot konnte nicht neben diesem Pfad erstellt werden: ",
    snapshot_write_failed: "Snapshot konnte nicht geschrieben werden: ",
    snapshot_install_failed: "Snapshot konnte nicht installiert werden: ",
    failed_to_open: "Öffnen fehlgeschlagen: ",
    top_level: "Oberste Ebene erreicht",
    entry_file_or_empty: "Eintrag ist eine Datei oder ein leeres Verzeichnis",
    gitignore_snapshot_unavailable: "Git-ignorierte Einträge sind für Snapshots nicht ermittelbar",
    scanning: "-> Scan läuft <-",
    snapshots_read_only: "Snapshots sind schreibgeschützt",
    traversal_running: "Scan läuft bereits",
    deleting_items: "Einträge werden gelöscht...",
    #[cfg(feature = "trash-move")]
    trashing_items: "Einträge werden in den Papierkorb verschoben...",
    no_cleanup_candidates: "Keine Bereinigungskandidaten in der Ansicht",
    cleanup_candidates_already_marked: "Bereinigungskandidaten sind bereits markiert",
    cleanup_detection_disabled: "Erkennung von Bereinigungskandidaten ist deaktiviert",
    no_gitignored_entries: "Keine Git-ignorierten Einträge in der Ansicht",
    gitignored_entries_already_marked: "Git-ignorierte Einträge sind bereits markiert",
    gitignore_detection_disabled: "Erkennung Git-ignorierter Einträge ist deaktiviert",
    no_match: "Keine Übereinstimmung gefunden",
    notification_scan: "Scan",
    notification_deletion: "Löschen",
    #[cfg(feature = "trash-move")]
    notification_trash: "Verschieben in den Papierkorb",
    notification_finished: "abgeschlossen",
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
    fn german_locale_selects_german_when_codeset_is_missing_or_utf8() {
        assert_eq!(detect([None, None, Some("de_DE.UTF-8")]), Language::German);
        assert_eq!(detect([None, None, Some("de")]), Language::German);
    }

    #[test]
    fn chinese_locale_selects_chinese_when_codeset_is_missing_or_utf8() {
        assert_eq!(detect([None, None, Some("zh_CN.UTF-8")]), Language::Chinese);
        assert_eq!(detect([None, None, Some("zh_SG.UTF8")]), Language::Chinese);
        assert_eq!(detect([None, None, Some("zh_Hans")]), Language::Chinese);
        assert_eq!(detect([None, None, Some("zh")]), Language::Chinese);
    }

    #[test]
    fn traditional_chinese_locales_are_english() {
        for locale in ["zh_TW.UTF-8", "zh_HK.UTF-8", "zh_MO.UTF-8", "zh_Hant"] {
            assert_eq!(detect([None, None, Some(locale)]), Language::English);
        }
    }

    #[test]
    fn explicit_non_utf8_supported_locales_are_english() {
        assert_eq!(
            detect([None, None, Some("de_DE.ISO-8859-1")]),
            Language::English
        );
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
    fn every_supported_language_has_interactive_ui_text() {
        assert_eq!(Language::English.ui_text().footer_sort_mode, "Sort mode");
        assert_eq!(Language::Japanese.ui_text().footer_sort_mode, "並べ替え");
        assert_eq!(Language::Korean.ui_text().footer_sort_mode, "정렬");
        assert_eq!(Language::Chinese.ui_text().footer_sort_mode, "排序");
        assert_eq!(Language::German.ui_text().footer_sort_mode, "Sortierung");
    }

    #[test]
    fn compact_action_labels_are_curated_per_language() {
        let labels = |language: Language| {
            let t = language.ui_text();
            (
                t.entries_mark_move,
                t.entries_mark_toggle,
                t.entries_cleanup,
                t.entries_gitignore,
                t.entries_all,
                t.glob_search,
                t.glob_case,
                t.glob_cancel,
                t.mark_toggle,
                t.mark_remove_all,
            )
        };

        assert_eq!(
            labels(Language::German),
            (
                "mark-move",
                "mark-toggle",
                "cleanup",
                "gitignore",
                "all",
                "Suche",
                "case",
                "cancel",
                "mark-toggle",
                "remove-all",
            )
        );
        assert_eq!(
            labels(Language::Japanese),
            (
                "マーク↓",
                "マーク切替",
                "整理",
                "Git無視",
                "全て",
                "検索",
                "大小",
                "取消",
                "マーク切替",
                "全解除",
            )
        );
        assert_eq!(
            labels(Language::Korean),
            (
                "표시↓",
                "표시전환",
                "정리",
                "Git무시",
                "전체",
                "검색",
                "대소문자",
                "취소",
                "표시전환",
                "모두해제",
            )
        );
        assert_eq!(
            labels(Language::Chinese),
            (
                "标记↓",
                "切换标记",
                "清理",
                "Git忽略",
                "全部",
                "搜索",
                "大小写",
                "取消",
                "切换标记",
                "全部移除",
            )
        );
    }

    #[test]
    fn dynamic_interface_messages_follow_each_language() {
        let expected = [
            (
                Language::English,
                "(4 visible, 43 total, 1.42 GB)",
                "Processed 7 entries in 0.23s  ",
                "Scan finished: 42 entries, 2.00 MB in 1.2s, 2 errors",
            ),
            (
                Language::Japanese,
                "(表示 4 件、合計 43 件、1.42 GB)",
                "7 件を 0.23s で処理  ",
                "スキャン完了: 42 件、2.00 MB、所要時間 1.2s、エラー 2 件",
            ),
            (
                Language::Korean,
                "(표시 4개, 전체 43개, 1.42 GB)",
                "7개 항목을 0.23s 동안 처리  ",
                "스캔 완료: 42개 항목, 2.00 MB, 소요 시간 1.2s, 오류 2건",
            ),
            (
                Language::Chinese,
                "(显示 4 项，共 43 项，1.42 GB)",
                "已处理 7 个条目，用时 0.23s  ",
                "扫描完成：42 个条目，2.00 MB，用时 1.2s，2 个错误",
            ),
            (
                Language::German,
                "(4 sichtbar, 43 gesamt, 1.42 GB)",
                "7 Einträge in 0.23s verarbeitet  ",
                "Scan abgeschlossen: 42 Einträge, 2.00 MB in 1.2s, 2 Fehler",
            ),
        ];

        for (language, statistics, progress, notification) in expected {
            assert_eq!(language.entries_statistics(4, "43", "1.42 GB"), statistics);
            assert_eq!(language.footer_progress(7, "0.23s"), progress);
            assert_eq!(
                language.notification_summary(
                    language.ui_text().notification_scan,
                    42,
                    "2.00 MB",
                    "1.2s",
                    2,
                ),
                notification
            );
        }
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
