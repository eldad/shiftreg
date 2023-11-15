#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

use flipperzero_rt::{entry, manifest};
use flipperzero_sys::{
    furi_hal_gpio_init_simple, furi_hal_gpio_write, GpioMode_GpioModeOutputPushPull, GpioPin,
};
use sys::random;

use core::ffi::CStr;

use flipperzero::{
    dialogs::{DialogMessage, DialogsApp},
    furi::thread::sleep,
    gui::canvas::Align,
};

use flipperzero_sys as sys;

// Define the FAP Manifest for this application
manifest!(
    name = "Shift Register",
    app_version = 1,
    has_icon = true,
    icon = "shiftreg.icon",
);
// Define the entry function
entry!(main);

/*
 *  App
 */

enum Action {
    Clear,
    ClearInverted,
    OutputEnableInverted,
    Serialize0,
    Serialize1,
    Latch,
}

#[allow(clippy::upper_case_acronyms)]
enum PinMapping {
    SER,
    OE,
    RCLK,
    SRCLK,
    SRCLR,
}

impl PinMapping {
    unsafe fn gpio(&self) -> &GpioPin {
        match self {
            PinMapping::SER => &sys::gpio_ext_pa7,
            PinMapping::OE => &sys::gpio_ext_pa6,
            PinMapping::RCLK => &sys::gpio_ext_pa4,
            PinMapping::SRCLK => &sys::gpio_ext_pb3,
            PinMapping::SRCLR => &sys::gpio_ext_pb2,
        }
    }
}

fn sleep_ms(t: u64) {
    sleep(core::time::Duration::from_millis(t));
}

unsafe fn shift_bit(state: bool) {
    let ser = PinMapping::SER.gpio();
    let clk = PinMapping::SRCLK.gpio();

    furi_hal_gpio_write(ser, state);
    sleep_ms(1);

    furi_hal_gpio_write(clk, true);
    sleep_ms(1);

    furi_hal_gpio_write(ser, false);
    sleep_ms(1);

    furi_hal_gpio_write(clk, false);
    sleep_ms(1);
}

unsafe fn clock_pin(pin: &GpioPin, state: bool) {
    furi_hal_gpio_write(pin, state);
    sleep_ms(1);

    furi_hal_gpio_write(pin, !state);
    sleep_ms(1);
}

impl Action {
    unsafe fn act(&self) {
        match self {
            Action::ClearInverted => furi_hal_gpio_write(PinMapping::SRCLR.gpio(), true),
            Action::Clear => clock_pin(PinMapping::SRCLR.gpio(), false),
            Action::OutputEnableInverted => furi_hal_gpio_write(PinMapping::OE.gpio(), false),
            Action::Serialize0 => shift_bit(false),
            Action::Serialize1 => shift_bit(true),
            Action::Latch => clock_pin(PinMapping::RCLK.gpio(), true),
        };
    }
}

enum AppMode {
    Manual,
    Auto,
    Immediate,
}

fn show_message(msgbytes: &[u8]) {
    let mut app = DialogsApp::open();
    let mut msg = DialogMessage::new();
    msg.set_header(
        CStr::from_bytes_with_nul(msgbytes).unwrap(),
        0,
        0,
        Align::Left,
        Align::Top,
    );
    app.show_message(&msg);
}

fn immediate_mode() {
    let mut app = DialogsApp::open();
    let mut msg = DialogMessage::new();
    msg.set_header(
        CStr::from_bytes_with_nul(b"Shift Register Control\0").unwrap(),
        0,
        0,
        Align::Left,
        Align::Top,
    );
    msg.set_text(
        CStr::from_bytes_with_nul(b"Immediate Mode\0").unwrap(),
        0,
        10,
        Align::Left,
        Align::Top,
    );
    msg.set_buttons(
        Some(CStr::from_bytes_with_nul(b"0\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"Clear\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"1\0").unwrap()),
    );

    loop {
        let button = app.show_message(&msg);

        let action = match button {
            flipperzero::dialogs::DialogMessageButton::Back => break,
            flipperzero::dialogs::DialogMessageButton::Left => Action::Serialize0,
            flipperzero::dialogs::DialogMessageButton::Right => Action::Serialize1,
            flipperzero::dialogs::DialogMessageButton::Center => Action::Clear,
        };

        unsafe {
            action.act();
            Action::Latch.act();
        }
    }
}

fn manual_mode() {
    let mut app = DialogsApp::open();
    let mut msg = DialogMessage::new();
    msg.set_header(
        CStr::from_bytes_with_nul(b"Shift Register Control\0").unwrap(),
        0,
        0,
        Align::Left,
        Align::Top,
    );
    msg.set_text(
        CStr::from_bytes_with_nul(b"Manual Mode\0").unwrap(),
        0,
        10,
        Align::Left,
        Align::Top,
    );
    msg.set_buttons(
        Some(CStr::from_bytes_with_nul(b"0\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"Latch\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"1\0").unwrap()),
    );

    loop {
        let button = app.show_message(&msg);

        let action = match button {
            flipperzero::dialogs::DialogMessageButton::Back => break,
            flipperzero::dialogs::DialogMessageButton::Left => Action::Serialize0,
            flipperzero::dialogs::DialogMessageButton::Right => Action::Serialize1,
            flipperzero::dialogs::DialogMessageButton::Center => Action::Latch,
        };

        unsafe {
            action.act();
        }
    }
}

unsafe fn auto_mode_pattern_1(t: u64) {
    for _ in 0..=7 {
        Action::Serialize1.act();
        Action::Latch.act();
        sleep_ms(t);
    }
    for _ in 0..=7 {
        Action::Serialize0.act();
        Action::Latch.act();
        sleep_ms(t);
    }
}

unsafe fn auto_mode_pattern_2(t: u64) {
    for i in 0..=15 {
        if i & 1 == 0 {
            Action::Serialize1.act();
        } else {
            Action::Serialize0.act();
        }
        Action::Latch.act();
        sleep_ms(t);
    }
}

unsafe fn auto_mode_random(t: u64) {
    let count_on = random() % 4 + 1;
    let count_off = random() % 4 + 1;

    let cycles = 3;

    for _ in 0..=cycles {
        for _ in 0..=count_on {
            Action::Serialize1.act();
            Action::Latch.act();
            sleep_ms(t);
        }
        for _ in 0..=count_off {
            Action::Serialize0.act();
            Action::Latch.act();
            sleep_ms(t);
        }
        for _ in 0..=count_on {
            Action::Serialize1.act();
            Action::Latch.act();
            sleep_ms(t);
        }
    }
    for _ in 0..=7 {
        Action::Serialize0.act();
        Action::Latch.act();
        sleep_ms(t);
    }
}

fn auto_mode() {
    unsafe {
        Action::Clear.act();

        for _ in 0..=2 {
            auto_mode_pattern_1(100);
            auto_mode_random(44);
        }
        for _ in 0..=2 {
            auto_mode_pattern_2(90);
            auto_mode_random(33);
        }
        for _ in 0..=2 {
            auto_mode_pattern_1(50);
            auto_mode_random(44);
            auto_mode_pattern_2(50);
            auto_mode_random(44);
        }
    }
}

// Entry point
fn main(_args: *mut u8) -> i32 {
    unsafe {
        furi_hal_gpio_init_simple(PinMapping::SER.gpio(), GpioMode_GpioModeOutputPushPull);
        furi_hal_gpio_init_simple(PinMapping::OE.gpio(), GpioMode_GpioModeOutputPushPull);
        furi_hal_gpio_init_simple(PinMapping::RCLK.gpio(), GpioMode_GpioModeOutputPushPull);
        furi_hal_gpio_init_simple(PinMapping::SRCLK.gpio(), GpioMode_GpioModeOutputPushPull);
        furi_hal_gpio_init_simple(PinMapping::SRCLR.gpio(), GpioMode_GpioModeOutputPushPull);

        Action::ClearInverted.act();
        Action::OutputEnableInverted.act();
    }

    let mut app = DialogsApp::open();
    let mut msg = DialogMessage::new();
    msg.set_header(
        CStr::from_bytes_with_nul(b"Shift Register Control\0").unwrap(),
        0,
        0,
        Align::Left,
        Align::Top,
    );
    msg.set_buttons(
        Some(CStr::from_bytes_with_nul(b"Manual\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"Auto\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"Instant\0").unwrap()),
    );

    loop {
        let button = app.show_message(&msg);

        let action = match button {
            flipperzero::dialogs::DialogMessageButton::Back => break,
            flipperzero::dialogs::DialogMessageButton::Left => AppMode::Manual,
            flipperzero::dialogs::DialogMessageButton::Right => AppMode::Immediate,
            flipperzero::dialogs::DialogMessageButton::Center => AppMode::Auto,
        };

        match action {
            AppMode::Manual => manual_mode(),
            AppMode::Auto => auto_mode(),
            AppMode::Immediate => immediate_mode(),
        }
    }

    show_message(b"Goodbye\0");

    0
}
