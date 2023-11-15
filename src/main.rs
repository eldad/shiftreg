#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

use flipperzero_rt::{entry, manifest};
use flipperzero_sys::{
    furi_hal_gpio_init_simple, furi_hal_gpio_write, GpioMode_GpioModeOutputPushPull, GpioPin,
};

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

unsafe fn shift_bit(state: bool) {
    let ser = PinMapping::SER.gpio();
    let clk = PinMapping::SRCLK.gpio();

    furi_hal_gpio_write(ser, state);
    sleep(core::time::Duration::from_millis(1));

    furi_hal_gpio_write(clk, true);
    sleep(core::time::Duration::from_millis(1));

    furi_hal_gpio_write(ser, false);
    sleep(core::time::Duration::from_millis(1));

    furi_hal_gpio_write(clk, false);
    sleep(core::time::Duration::from_millis(1));
}

unsafe fn clock_pin(pin: &GpioPin, state: bool) {
    furi_hal_gpio_write(pin, state);
    sleep(core::time::Duration::from_millis(1));

    furi_hal_gpio_write(pin, !state);
    sleep(core::time::Duration::from_millis(1));
}

impl Action {
    unsafe fn act(&self) {
        match self {
            Action::ClearInverted => furi_hal_gpio_write(PinMapping::SRCLR.gpio(), true),
            Action::OutputEnableInverted => furi_hal_gpio_write(PinMapping::OE.gpio(), false),
            Action::Serialize0 => shift_bit(false),
            Action::Serialize1 => shift_bit(true),
            Action::Latch => clock_pin(PinMapping::RCLK.gpio(), true),
        };
    }
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
        Some(CStr::from_bytes_with_nul(b"0\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"Latch\0").unwrap()),
        Some(CStr::from_bytes_with_nul(b"1\0").unwrap()),
    );

    loop {
        let button = app.show_message(&msg);

        let action = match button {
            flipperzero::dialogs::DialogMessageButton::Back => {
                show_message(b"Goodbye\0");
                break;
            }
            flipperzero::dialogs::DialogMessageButton::Left => Action::Serialize0,
            flipperzero::dialogs::DialogMessageButton::Right => Action::Serialize1,
            flipperzero::dialogs::DialogMessageButton::Center => Action::Latch,
        };

        unsafe {
            action.act();
        }
    }

    0
}
