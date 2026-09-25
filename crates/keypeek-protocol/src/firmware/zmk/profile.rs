use std::sync::OnceLock;

use crate::firmware::zmk::ZmkKeyPresenter;
use keypeek_core::keymap_editor::catalog::common;
use keypeek_core::keymap_editor::draft::EditorSection;
use keypeek_core::keymap_editor::profile::{EditorProfile, SidebarSection};
use keypeek_core::keymap_editor::{CandidateGroup, LayerTapTarget};
use keypeek_core::{
    BacklightAction, BluetoothAction, KeyPresenter, KeySpec, LayerActivation, LayoutKey,
    MouseAction, MouseButton, OutputTarget, PowerAction, RgbAction,
};

const ZMK_SIDEBAR_SECTIONS: [SidebarSection<EditorSection>; 5] = [
    SidebarSection {
        title: "Keys",
        items: &[EditorSection::Keyboard, EditorSection::KeyToggle],
    },
    SidebarSection {
        title: "Layers & Mods",
        items: &[
            EditorSection::Layers,
            EditorSection::Combo,
            EditorSection::OneShot,
            EditorSection::ModTap,
        ],
    },
    SidebarSection {
        title: "Wireless",
        items: &[EditorSection::Bluetooth, EditorSection::Output],
    },
    SidebarSection {
        title: "Lighting",
        items: &[EditorSection::Backlight, EditorSection::Rgb],
    },
    SidebarSection {
        title: "Other",
        items: &[EditorSection::Mouse, EditorSection::Custom],
    },
];

#[derive(Debug, Default, Clone, Copy)]
pub struct ZmkEditorProfile;

fn zmk_keyboard_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        let usages = super::codec::zmk_all_keyboard_usages();
        let candidates = common::build_keyboard_candidates(&ZmkKeyPresenter, usages, |id| {
            super::codec::zmk_search_tokens_for_hid(0x07, id)
        });
        CandidateGroup {
            name: "Basic",
            candidates,
        }
    })
}

fn zmk_media_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        let usages = super::codec::zmk_all_consumer_usages();
        let candidates = common::build_media_candidates(&ZmkKeyPresenter, usages, |id| {
            super::codec::zmk_search_tokens_for_hid(0x0C, id)
        });
        CandidateGroup {
            name: "Media",
            candidates,
        }
    })
}

fn zmk_apps_and_web_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        let usages = super::codec::zmk_all_consumer_usages();
        let candidates = common::build_app_and_web_candidates(&ZmkKeyPresenter, usages, |id| {
            super::codec::zmk_search_tokens_for_hid(0x0C, id)
        });
        CandidateGroup {
            name: "Apps & Web",
            candidates,
        }
    })
}

fn zmk_international_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        let usages = super::codec::zmk_all_keyboard_usages();
        let candidates = common::build_international_candidates(&ZmkKeyPresenter, usages, |id| {
            super::codec::zmk_search_tokens_for_hid(0x07, id)
        });
        CandidateGroup {
            name: "International",
            candidates,
        }
    })
}

const ZMK_BOOT_POWER_ACTIONS: [(PowerAction, &[&str]); 7] = [
    (PowerAction::Reset, &["reset", "reboot", "sys_reset"]),
    (
        PowerAction::Bootloader,
        &["bootloader", "dfu", "flash", "boot"],
    ),
    (PowerAction::SoftOff, &["soft off", "power off", "shutdown"]),
    (
        PowerAction::UnlockKeymap,
        &["unlock", "keymap unlock", "studio unlock"],
    ),
    (PowerAction::Toggle, &["ext pwr tog", "power toggle"]),
    (PowerAction::On, &["ext pwr on", "power on"]),
    (PowerAction::Off, &["ext pwr off", "power off"]),
];

fn zmk_system_and_power_group() -> &'static CandidateGroup {
    static GROUP: OnceLock<CandidateGroup> = OnceLock::new();
    GROUP.get_or_init(|| {
        common::build_system_and_power_group(
            &ZmkKeyPresenter,
            &ZMK_BOOT_POWER_ACTIONS,
            |_| &[],
            |act| match act {
                PowerAction::Reset => &["&sys_reset"],
                PowerAction::Bootloader => &["&bootloader"],
                PowerAction::SoftOff => &["&soft_off"],
                PowerAction::UnlockKeymap => &["&studio_unlock"],
                PowerAction::Toggle => &["&ext_power EP_TOG"],
                PowerAction::On => &["&ext_power EP_ON"],
                PowerAction::Off => &["&ext_power EP_OFF"],
                _ => &[],
            },
        )
    })
}

fn zmk_tap_categories() -> &'static [CandidateGroup] {
    static CATEGORIES: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    CATEGORIES.get_or_init(|| {
        vec![
            zmk_keyboard_group().clone(),
            zmk_media_group().clone(),
            zmk_apps_and_web_group().clone(),
            zmk_special_groups()[0].clone(),
            zmk_system_and_power_group().clone(),
            zmk_international_group().clone(),
        ]
    })
}

fn zmk_bluetooth_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_bluetooth_group(
            &ZmkKeyPresenter,
            |act| -> &'static [&'static str] {
                match act {
                    BluetoothAction::Clear => &["&bt BT_CLR", "bt_clr"],
                    BluetoothAction::Next => &["&bt BT_NXT", "bt_nxt"],
                    BluetoothAction::Prev => &["&bt BT_PRV", "bt_prv"],
                    BluetoothAction::ClearAll => &["&bt BT_CLR_ALL", "bt_clr_all"],
                    BluetoothAction::Select(_) => &["&bt BT_SEL", "bt_sel"],
                    BluetoothAction::Disconnect(_) => &["&bt BT_DISC", "bt_disc"],
                    _ => &[],
                }
            },
        )]
    })
}

fn zmk_output_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_output_group(
            &ZmkKeyPresenter,
            |target| -> &'static [&'static str] {
                match target {
                    OutputTarget::Toggle => &["&out OUT_TOG", "out_tog"],
                    OutputTarget::Usb => &["&out OUT_USB", "out_usb"],
                    OutputTarget::Ble => &["&out OUT_BLE", "out_ble"],
                    OutputTarget::None => &["&out OUT_NONE", "out_none"],
                    _ => &[],
                }
            },
        )]
    })
}

fn zmk_backlight_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_backlight_group(
            &ZmkKeyPresenter,
            |act| -> &'static [&'static str] {
                match act {
                    BacklightAction::Toggle => &["&bl BL_TOG"],
                    BacklightAction::On => &["&bl BL_ON"],
                    BacklightAction::Off => &["&bl BL_OFF"],
                    BacklightAction::Inc => &["&bl BL_INC"],
                    BacklightAction::Dec => &["&bl BL_DEC"],
                    BacklightAction::Cycle => &["&bl BL_CYCLE"],
                    _ => &[],
                }
            },
        )]
    })
}

fn zmk_rgb_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_rgb_underglow_group(
            &ZmkKeyPresenter,
            |act| -> &'static [&'static str] {
                match act {
                    RgbAction::Toggle => &["&rgb_ug RGB_TOG"],
                    RgbAction::On => &["&rgb_ug RGB_ON"],
                    RgbAction::Off => &["&rgb_ug RGB_OFF"],
                    RgbAction::EffectInc => &["&rgb_ug RGB_EFF"],
                    RgbAction::EffectDec => &["&rgb_ug RGB_EFR"],
                    RgbAction::HueInc => &["&rgb_ug RGB_HUI"],
                    RgbAction::HueDec => &["&rgb_ug RGB_HUD"],
                    RgbAction::SatInc => &["&rgb_ug RGB_SAI"],
                    RgbAction::SatDec => &["&rgb_ug RGB_SAD"],
                    RgbAction::BrightInc => &["&rgb_ug RGB_BRI"],
                    RgbAction::BrightDec => &["&rgb_ug RGB_BRD"],
                    RgbAction::SpeedInc => &["&rgb_ug RGB_SPI"],
                    RgbAction::SpeedDec => &["&rgb_ug RGB_SPD"],
                    _ => &[],
                }
            },
        )]
    })
}

fn zmk_mouse_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        common::build_mouse_groups(&ZmkKeyPresenter, false, |spec| -> &'static [&'static str] {
            match spec {
                KeySpec::Mouse(MouseAction::Press(MouseButton::Left)) => &["&mkp LCLK"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Right)) => &["&mkp RCLK"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Middle)) => &["&mkp MCLK"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Button4)) => &["&mkp MB4"],
                KeySpec::Mouse(MouseAction::Press(MouseButton::Button5)) => &["&mkp MB5"],
                KeySpec::Mouse(MouseAction::Move { x: 0, y: -1 }) => &["&mmv MOVE_UP"],
                KeySpec::Mouse(MouseAction::Move { x: 0, y: 1 }) => &["&mmv MOVE_DOWN"],
                KeySpec::Mouse(MouseAction::Move { x: -1, y: 0 }) => &["&mmv MOVE_LEFT"],
                KeySpec::Mouse(MouseAction::Move { x: 1, y: 0 }) => &["&mmv MOVE_RIGHT"],
                KeySpec::Mouse(MouseAction::Scroll { x: 0, y: 1 }) => &["&msc SCRL_UP"],
                KeySpec::Mouse(MouseAction::Scroll { x: 0, y: -1 }) => &["&msc SCRL_DOWN"],
                _ => &[],
            }
        })
    })
}

fn zmk_special_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        vec![common::build_special_group(
            &ZmkKeyPresenter,
            |spec| -> &'static [&'static str] {
                match spec {
                    KeySpec::Transparent => &["&trans"],
                    KeySpec::None => &["&none"],
                    KeySpec::CapsWord => &["&caps_word"],
                    KeySpec::KeyRepeat => &["&key_repeat"],
                    KeySpec::GraveEscape => &["&gresc", "gresc"],
                    KeySpec::KeyPress { key, .. } => match key.id {
                        0x82 => &["&kp LCAPS", "LCAPS"],
                        0x83 => &["&kp LNUM", "LNUM"],
                        0x84 => &["&kp LSCRL", "LSCRL"],
                        0x7A => &["&kp K_UNDO"],
                        0x7B => &["&kp K_CUT"],
                        0x7C => &["&kp K_COPY"],
                        0x7D => &["&kp K_PASTE"],
                        0x7E => &["&kp K_FIND"],
                        0x79 => &["&kp K_AGAIN"],
                        0x78 => &["&kp K_STOP"],
                        0x66 => &["&kp K_POWER"],
                        0x68 => &["&kp F13"],
                        0x69 => &["&kp F14"],
                        0x6A => &["&kp F15"],
                        0x6B => &["&kp F16"],
                        0x6C => &["&kp F17"],
                        0x6D => &["&kp F18"],
                        0x6E => &["&kp F19"],
                        0x6F => &["&kp F20"],
                        0x70 => &["&kp F21"],
                        0x71 => &["&kp F22"],
                        0x72 => &["&kp F23"],
                        0x73 => &["&kp F24"],
                        _ => &[],
                    },
                    _ => &[],
                }
            },
        )]
    })
}

fn zmk_custom_groups() -> &'static [CandidateGroup] {
    static GROUPS: OnceLock<Vec<CandidateGroup>> = OnceLock::new();
    GROUPS.get_or_init(|| common::build_custom_groups(&ZmkKeyPresenter))
}

const ZMK_LAYER_OPS: [(&str, LayerActivation, &[&str]); 4] = [
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
];

impl KeyPresenter for ZmkEditorProfile {
    fn present_key(&self, spec: &KeySpec, layer_names: &[String]) -> Option<LayoutKey> {
        ZmkKeyPresenter.present_key(spec, layer_names)
    }
}

impl EditorProfile for ZmkEditorProfile {
    fn presenter(&self) -> &dyn KeyPresenter {
        self
    }

    fn name(&self) -> &'static str {
        "ZMK"
    }

    fn sidebar_sections(&self) -> &[SidebarSection<EditorSection>] {
        &ZMK_SIDEBAR_SECTIONS
    }

    fn tap_categories(&self) -> &'static [CandidateGroup] {
        zmk_tap_categories()
    }

    fn section_groups(&self, section: EditorSection) -> &'static [CandidateGroup] {
        match section {
            EditorSection::Keyboard
            | EditorSection::KeyToggle
            | EditorSection::Combo
            | EditorSection::ModTap
            | EditorSection::OneShot => zmk_tap_categories(),
            EditorSection::Bluetooth => zmk_bluetooth_groups(),
            EditorSection::Output => zmk_output_groups(),
            EditorSection::Backlight => zmk_backlight_groups(),
            EditorSection::Rgb => zmk_rgb_groups(),
            EditorSection::Mouse => zmk_mouse_groups(),
            EditorSection::Custom => zmk_custom_groups(),
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
            &ZmkKeyPresenter,
            layer_count,
            layer_names,
            tap,
            &ZMK_LAYER_OPS,
            |act| match act {
                LayerActivation::Momentary => &["&mo"][..],
                LayerActivation::Toggle => &["&tog"][..],
                LayerActivation::To => &["&to"][..],
                LayerActivation::Sticky => &["&sl"][..],
                _ => &[][..],
            },
        )
    }

    fn supports_tap_modifiers(&self) -> bool {
        true
    }

    fn supports_oneshot_keys(&self) -> bool {
        true
    }
}
