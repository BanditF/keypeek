use std::sync::OnceLock;

use crate::firmware::qmk::QmkKeyPresenter;
use keypeek_core::keymap_editor::catalog::common;
use keypeek_core::keymap_editor::draft::EditorSection;
use keypeek_core::keymap_editor::profile::{EditorProfile, SidebarSection};
use keypeek_core::keymap_editor::{Candidate, CandidateGroup, LayerTapTarget};
use keypeek_core::{
    AudioAction, BacklightAction, CustomBinding, CustomKind, KeyPresenter, KeySpec,
    LayerActivation, LayoutKey, MouseAction, MouseButton, PowerAction, RgbAction, RgbMatrixAction,
};
use qmk_via_api::keycodes::Keycode;

const QMK_SIDEBAR_SECTIONS: [SidebarSection<EditorSection>; 4] = [
    SidebarSection {
        title: "Keys",
        items: &[EditorSection::Keyboard, EditorSection::Special],
    },
    SidebarSection {
        title: "Layers & Mods",
        items: &[
            EditorSection::Layers,
            EditorSection::Combo,
            EditorSection::OneShot,
            EditorSection::ModTap,
            EditorSection::LayerMod,
        ],
    },
    SidebarSection {
        title: "Lighting & Audio",
        items: &[
            EditorSection::Backlight,
            EditorSection::Rgb,
            EditorSection::RgbMatrix,
            EditorSection::Audio,
        ],
    },
    SidebarSection {
        title: "Other",
        items: &[
            EditorSection::Mouse,
            EditorSection::Custom,
            EditorSection::RawHex,
        ],
    },
];

#[derive(Debug, Default, Clone, Copy)]
pub struct QmkEditorProfile;

fn qmk_keyboard_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        let usages = super::codec::qmk_all_basic_usages();
        let candidates = common::build_keyboard_candidates(&QmkKeyPresenter, usages, |id| {
            super::codec::qmk_search_tokens_for_hid(0x07, id)
        });
        CandidateGroup {
            name: "Basic",
            candidates,
        }
    })
}

fn qmk_media_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        let usages = super::codec::qmk_all_media_usages();
        let candidates = common::build_media_candidates(&QmkKeyPresenter, usages, |id| {
            super::codec::qmk_search_tokens_for_hid(0x0C, id)
        });
        CandidateGroup {
            name: "Media",
            candidates,
        }
    })
}

fn qmk_apps_and_web_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        let usages = super::codec::qmk_all_media_usages();
        let candidates = common::build_app_and_web_candidates(&QmkKeyPresenter, usages, |id| {
            super::codec::qmk_search_tokens_for_hid(0x0C, id)
        });
        CandidateGroup {
            name: "Apps & Web",
            candidates,
        }
    })
}

fn qmk_international_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        let usages = super::codec::qmk_all_basic_usages();
        let candidates = common::build_international_candidates(&QmkKeyPresenter, usages, |id| {
            super::codec::qmk_search_tokens_for_hid(0x07, id)
        });
        CandidateGroup {
            name: "International",
            candidates,
        }
    })
}

const QMK_BOOT_POWER_ACTIONS: [(PowerAction, &[&str]); 3] = [
    (PowerAction::Reset, &["reset", "reboot"]),
    (
        PowerAction::Bootloader,
        &["bootloader", "dfu", "flash", "boot"],
    ),
    (PowerAction::Other(0xEE), &["clear eeprom", "eeprom reset"]),
];

fn qmk_system_and_power_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        common::build_system_and_power_group(
            &QmkKeyPresenter,
            &QMK_BOOT_POWER_ACTIONS,
            |id| match id {
                0x81 => &["KC_SYSTEM_POWER"][..],
                0x82 => &["KC_SYSTEM_SLEEP"][..],
                0x83 => &["KC_SYSTEM_WAKE"][..],
                _ => &[][..],
            },
            |act| match act {
                PowerAction::Reset => &["QK_BOOT", "QK_REBOOT"],
                PowerAction::Bootloader => &["QK_BOOTLOADER"],
                PowerAction::Other(0xEE) => &["QK_CLEAR_EEPROM"],
                _ => &[],
            },
        )
    })
}

fn qmk_tap_categories() -> &'static [CandidateGroup] {
    static CATEGORIES: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    CATEGORIES.get_or_init(|| {
        vec![
            qmk_keyboard_group().clone(),
            qmk_media_group().clone(),
            qmk_apps_and_web_group().clone(),
            qmk_special_groups()[0].clone(),
            qmk_system_and_power_group().clone(),
            qmk_international_group().clone(),
        ]
    })
}

fn qmk_backlight_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_backlight_group(
            &QmkKeyPresenter,
            common::ALL_BACKLIGHT_ACTIONS,
            |act| -> &'static [&'static str] {
                match act {
                    BacklightAction::Toggle => &["BL_TOGG", "QK_BACKLIGHT_TOGGLE"],
                    BacklightAction::On => &["BL_ON", "QK_BACKLIGHT_ON"],
                    BacklightAction::Off => &["BL_OFF", "QK_BACKLIGHT_OFF"],
                    BacklightAction::Inc => &["BL_UP", "QK_BACKLIGHT_UP"],
                    BacklightAction::Dec => &["BL_DOWN", "QK_BACKLIGHT_DOWN"],
                    BacklightAction::Cycle => &["BL_STEP", "QK_BACKLIGHT_STEP"],
                    BacklightAction::BreathingToggle => {
                        &["BL_BRTG", "QK_BACKLIGHT_TOGGLE_BREATHING"]
                    }
                    _ => &[],
                }
            },
        )]
    })
}

fn qmk_rgb_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_rgb_underglow_group(
            &QmkKeyPresenter,
            common::QMK_RGB_UNDERGLOW_ACTIONS,
            |act| -> &'static [&'static str] {
                match act {
                    RgbAction::Toggle => &["RGB_TOG", "QK_UNDERGLOW_TOGGLE"],
                    RgbAction::EffectInc => &["RGB_MOD", "QK_UNDERGLOW_MODE_NEXT"],
                    RgbAction::EffectDec => &["RGB_RMOD", "QK_UNDERGLOW_MODE_PREVIOUS"],
                    RgbAction::HueInc => &["RGB_HUI", "QK_UNDERGLOW_HUE_UP"],
                    RgbAction::HueDec => &["RGB_HUD", "QK_UNDERGLOW_HUE_DOWN"],
                    RgbAction::SatInc => &["RGB_SAI", "QK_UNDERGLOW_SATURATION_UP"],
                    RgbAction::SatDec => &["RGB_SAD", "QK_UNDERGLOW_SATURATION_DOWN"],
                    RgbAction::BrightInc => &["RGB_VAI", "QK_UNDERGLOW_VALUE_UP"],
                    RgbAction::BrightDec => &["RGB_VAD", "QK_UNDERGLOW_VALUE_DOWN"],
                    RgbAction::SpeedInc => &["RGB_SPI", "QK_UNDERGLOW_SPEED_UP"],
                    RgbAction::SpeedDec => &["RGB_SPD", "QK_UNDERGLOW_SPEED_DOWN"],
                    _ => &[],
                }
            },
        )]
    })
}

fn qmk_rgb_matrix_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_rgb_matrix_group(
            &QmkKeyPresenter,
            |act| -> &'static [&'static str] {
                match act {
                    RgbMatrixAction::On => &["RM_ON", "QK_RGB_MATRIX_ON"],
                    RgbMatrixAction::Off => &["RM_OFF", "QK_RGB_MATRIX_OFF"],
                    RgbMatrixAction::Toggle => &["RM_TOGG", "QK_RGB_MATRIX_TOGGLE"],
                    RgbMatrixAction::ModeNext => &["RM_NEXT", "QK_RGB_MATRIX_MODE_NEXT"],
                    RgbMatrixAction::ModePrev => &["RM_PREV", "QK_RGB_MATRIX_MODE_PREVIOUS"],
                    RgbMatrixAction::HueInc => &["RM_HUEU", "QK_RGB_MATRIX_HUE_UP"],
                    RgbMatrixAction::HueDec => &["RM_HUED", "QK_RGB_MATRIX_HUE_DOWN"],
                    RgbMatrixAction::SatInc => &["RM_SATU", "QK_RGB_MATRIX_SATURATION_UP"],
                    RgbMatrixAction::SatDec => &["RM_SATD", "QK_RGB_MATRIX_SATURATION_DOWN"],
                    RgbMatrixAction::BrightInc => &["RM_VALU", "QK_RGB_MATRIX_VALUE_UP"],
                    RgbMatrixAction::BrightDec => &["RM_VALD", "QK_RGB_MATRIX_VALUE_DOWN"],
                    RgbMatrixAction::SpeedInc => &["RM_SPDU", "QK_RGB_MATRIX_SPEED_UP"],
                    RgbMatrixAction::SpeedDec => &["RM_SPDD", "QK_RGB_MATRIX_SPEED_DOWN"],
                    _ => &[],
                }
            },
        )]
    })
}

fn qmk_audio_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_audio_group(
            &QmkKeyPresenter,
            |act| -> &'static [&'static str] {
                match act {
                    AudioAction::On => &["QK_AUDIO_ON"],
                    AudioAction::Off => &["QK_AUDIO_OFF"],
                    AudioAction::Toggle => &["QK_AUDIO_TOGGLE"],
                    AudioAction::ClickyToggle => &["QK_AUDIO_CLICKY_TOGGLE"],
                    AudioAction::ClickyOn => &["QK_AUDIO_CLICKY_ON"],
                    AudioAction::ClickyOff => &["QK_AUDIO_CLICKY_OFF"],
                    AudioAction::ClickyUp => &["QK_AUDIO_CLICKY_UP"],
                    AudioAction::ClickyDown => &["QK_AUDIO_CLICKY_DOWN"],
                    AudioAction::ClickyReset => &["QK_AUDIO_CLICKY_RESET"],
                    AudioAction::MusicOn => &["QK_MUSIC_ON"],
                    AudioAction::MusicOff => &["QK_MUSIC_OFF"],
                    AudioAction::MusicToggle => &["QK_MUSIC_TOGGLE"],
                    AudioAction::MusicModeNext => &["QK_MUSIC_MODE_NEXT"],
                    AudioAction::VoiceNext => &["QK_AUDIO_VOICE_NEXT"],
                    AudioAction::VoicePrev => &["QK_AUDIO_VOICE_PREVIOUS"],
                    _ => &[],
                }
            },
        )]
    })
}

fn qmk_mouse_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        common::build_mouse_groups(&QmkKeyPresenter, true, |spec| -> &'static [&'static str] {
            match spec {
                KeySpec::Mouse(MouseAction::Press(MouseButton::Left)) => &["MS_BTN1"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Right)) => &["MS_BTN2"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Middle)) => &["MS_BTN3"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Button4)) => &["MS_BTN4"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Button5)) => &["MS_BTN5"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Other(6))) => &["MS_BTN6"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Other(7))) => &["MS_BTN7"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Other(8))) => &["MS_BTN8"],
                KeySpec::Mouse(MouseAction::Move { x: 0, y: -1 }) => &["MS_UP"],
                KeySpec::Mouse(MouseAction::Move { x: 0, y: 1 }) => &["MS_DOWN"],
                KeySpec::Mouse(MouseAction::Move { x: -1, y: 0 }) => &["MS_LEFT"],
                KeySpec::Mouse(MouseAction::Move { x: 1, y: 0 }) => &["MS_RIGHT"],
                KeySpec::Mouse(MouseAction::Scroll { x: 0, y: 1 }) => &["MS_WH_UP"],
                KeySpec::Mouse(MouseAction::Scroll { x: 0, y: -1 }) => &["MS_WH_DOWN"],
                KeySpec::Mouse(MouseAction::Acceleration(0)) => &["MS_ACL0"],
                KeySpec::Mouse(MouseAction::Acceleration(1)) => &["MS_ACL1"],
                KeySpec::Mouse(MouseAction::Acceleration(2)) => &["MS_ACL2"],
                _ => &[],
            }
        })
    })
}

fn qmk_special_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_special_group(
            &QmkKeyPresenter,
            |spec| -> &'static [&'static str] {
                match spec {
                    KeySpec::Transparent => &["KC_TRNS"],
                    KeySpec::None => &["KC_NO"],
                    KeySpec::CapsWord => &["QK_CAPS_WORD_TOGGLE", "CW_TOGG"],
                    KeySpec::KeyRepeat => &["QK_KEY_REPEAT", "QK_REP"],
                    KeySpec::AltKeyRepeat => &["QK_ALT_REPEAT_KEY", "QK_AREP"],
                    KeySpec::KeyLock => &["QK_LOCK"],
                    KeySpec::GraveEscape => &["QK_GRAVE_ESCAPE", "QK_GESC"],
                    KeySpec::KeyPress { key, .. } => match key.id {
                        0x82 => &["KC_LOCKING_CAPS_LOCK", "KC_LCAP"],
                        0x83 => &["KC_LOCKING_NUM_LOCK", "KC_LNUM"],
                        0x84 => &["KC_LOCKING_SCROLL_LOCK", "KC_LSCR"],
                        0x7A => &["KC_UNDO"],
                        0x7B => &["KC_CUT"],
                        0x7C => &["KC_COPY"],
                        0x7D => &["KC_PASTE"],
                        0x7E => &["KC_FIND"],
                        0x79 => &["KC_AGAIN"],
                        0x78 => &["KC_STOP"],
                        0x66 => &["KC_POWER", "KC_PWR"],
                        0x68 => &["KC_F13"],
                        0x69 => &["KC_F14"],
                        0x6A => &["KC_F15"],
                        0x6B => &["KC_F16"],
                        0x6C => &["KC_F17"],
                        0x6D => &["KC_F18"],
                        0x6E => &["KC_F19"],
                        0x6F => &["KC_F20"],
                        0x70 => &["KC_F21"],
                        0x71 => &["KC_F22"],
                        0x72 => &["KC_F23"],
                        0x73 => &["KC_F24"],
                        _ => &[],
                    },
                    _ => &[],
                }
            },
        )]
    })
}

fn qmk_custom_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        let mut groups = common::build_custom_groups(&QmkKeyPresenter);
        let dm_keys = [
            (
                Keycode::QK_DYNAMIC_MACRO_RECORD_START_1 as u16,
                &["DM_REC1", "DM Rec 1", "dm record 1"][..],
            ),
            (
                Keycode::QK_DYNAMIC_MACRO_RECORD_START_2 as u16,
                &["DM_REC2", "DM Rec 2", "dm record 2"][..],
            ),
            (
                Keycode::QK_DYNAMIC_MACRO_RECORD_STOP as u16,
                &["DM_RSTP", "DM Stop", "dm stop"][..],
            ),
            (
                Keycode::QK_DYNAMIC_MACRO_PLAY_1 as u16,
                &["DM_PLY1", "DM Play 1", "dm play 1"][..],
            ),
            (
                Keycode::QK_DYNAMIC_MACRO_PLAY_2 as u16,
                &["DM_PLY2", "DM Play 2", "dm play 2"][..],
            ),
        ];

        let dm_candidates: Vec<Candidate> = dm_keys
            .into_iter()
            .map(|(code, tokens)| {
                let name = super::keycode_labels::try_resolve_qmk_key(code)
                    .map(|lk| lk.tap.full)
                    .unwrap_or_else(|| format!("0x{code:04X}"));
                let spec = KeySpec::Custom(CustomBinding {
                    kind: CustomKind::Raw,
                    id: code as u32,
                    name: Some(name),
                    param1: None,
                    param2: None,
                });
                let mut cand = Candidate::from_action(spec, &QmkKeyPresenter, &[]);
                for &token in tokens {
                    cand = cand.with_search_token(token);
                }
                cand
            })
            .collect();

        groups.push(CandidateGroup {
            name: "Dynamic Macro",
            candidates: dm_candidates,
        });

        groups
    })
}

const QMK_LAYER_OPS: [(&str, LayerActivation, &[&str]); 6] = [
    (
        "Momentary",
        LayerActivation::Momentary,
        &["mo", "momentary"],
    ),
    ("Toggle", LayerActivation::Toggle, &["tg", "toggle"]),
    ("Switch To Layer", LayerActivation::To, &["to", "switch"]),
    (
        "Sticky Layer",
        LayerActivation::Sticky,
        &["sl", "sticky", "oneshot"],
    ),
    (
        "Set Default Layer",
        LayerActivation::Default,
        &["df", "default"],
    ),
    (
        "Tap Toggle",
        LayerActivation::TapToggle,
        &["tt", "tap toggle"],
    ),
];

impl KeyPresenter for QmkEditorProfile {
    fn present_key(&self, spec: &KeySpec, layer_names: &[String]) -> Option<LayoutKey> {
        QmkKeyPresenter.present_key(spec, layer_names)
    }
}

impl EditorProfile for QmkEditorProfile {
    fn presenter(&self) -> &dyn KeyPresenter {
        self
    }

    fn name(&self) -> &'static str {
        "QMK"
    }

    fn sidebar_sections(&self) -> &[SidebarSection<EditorSection>] {
        &QMK_SIDEBAR_SECTIONS
    }

    fn section_label(&self, section: EditorSection) -> &'static str {
        match section {
            EditorSection::OneShot => "One-Shot Mod",
            _ => section.label(),
        }
    }

    fn tap_categories(&self) -> &'static [CandidateGroup] {
        qmk_tap_categories()
    }

    fn section_groups(&self, section: EditorSection) -> &'static [CandidateGroup] {
        match section {
            EditorSection::Keyboard
            | EditorSection::Combo
            | EditorSection::ModTap
            | EditorSection::OneShot => qmk_tap_categories(),
            EditorSection::Special => qmk_special_groups(),
            EditorSection::Backlight => qmk_backlight_groups(),
            EditorSection::Rgb => qmk_rgb_groups(),
            EditorSection::RgbMatrix => qmk_rgb_matrix_groups(),
            EditorSection::Audio => qmk_audio_groups(),
            EditorSection::Mouse => qmk_mouse_groups(),
            EditorSection::Custom => qmk_custom_groups(),
            _ => &[],
        }
    }

    fn layer_groups(
        &self,
        layer_count: usize,
        layer_names: &[String],
        tap: LayerTapTarget,
    ) -> Vec<CandidateGroup> {
        common::build_layer_groups(
            &QmkKeyPresenter,
            layer_count,
            layer_names,
            tap,
            &QMK_LAYER_OPS,
            |_| &[],
        )
    }

    fn parse_raw_keycode(&self, raw: &str) -> Option<KeySpec> {
        let code = u16::from_str_radix(raw, 16).ok()?;
        Some(super::codec::qmk_to_keyspec(code))
    }
}
