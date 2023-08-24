#![no_main]
#![no_std]

use keyberon::action::Action::{self, *};
use keyberon::action::{k, l, m, HoldTapAction, HoldTapConfig};
use keyberon::debounce::Debouncer;
use keyberon::hid::HidClass;
use keyberon::key_code::KeyCode::*;
use keyberon::key_code::{KbHidReport, KeyCode};
use keyberon::keyboard::Keyboard;
use keyberon::layout::Layout;
use keyberon::matrix::Matrix;
use panic_halt as _;
use rtic::app;
use stm32f1xx_hal::gpio::{EPin, Input, Output, PullUp, PushPull};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32f1xx_hal::{gpio, pac, timer, usb};
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;

type UsbClass = keyberon::Class<'static, UsbBusType, Leds>;
type UsbDevice = usb_device::device::UsbDevice<'static, UsbBusType>;

pub struct Leds {
    caps_lock: gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>,
}
impl keyberon::keyboard::Leds for Leds {
    fn caps_lock(&mut self, status: bool) {
        if status {
            self.caps_lock.set_low()
        } else {
            self.caps_lock.set_high()
        }
    }
}

const CUT: Action = m(&[LShift, Delete].as_slice());
const COPY: Action = m(&[LCtrl, Insert].as_slice());
const PASTE: Action = m(&[LShift, Insert].as_slice());
const L2_ENTER: Action = HoldTap(&HoldTapAction {
    timeout: 200,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    tap_hold_interval: 0,
    hold: l(2),
    tap: k(Enter),
});
const L1_SP: Action = HoldTap(&HoldTapAction {
    timeout: 200,
    config: HoldTapConfig::Default,
    tap_hold_interval: 0,
    hold: l(1),
    tap: k(Space),
});
const CSPACE: Action = m(&[LCtrl, Space].as_slice());
macro_rules! s {
    ($k:ident) => {
        m(&[LShift, $k].as_slice())
    };
}
macro_rules! a {
    ($k:ident) => {
        m(&[RAlt, $k].as_slice())
    };
}

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers<12, 5, 4> = [
      [
        [k(Grave),  k(Kb1),k(Kb2),k(Kb3),  k(Kb4),k(Kb5), k(Kb6),   k(Kb7),  k(Kb8), k(Kb9),  k(Kb0),   k(Minus)   ],
        [k(Tab),     k(Q), k(W),  k(E),    k(R), k(T),    k(Y),     k(U),    k(I),   k(O),    k(P),     k(LBracket)],
        [k(RBracket),k(A), k(S),  k(D),    k(F), k(G),    k(H),     k(J),    k(K),   k(L),    k(SColon),k(Quote)   ],
        [k(Equal),   k(Z), k(X),  k(C),    k(V), k(B),    k(N),     k(M),    k(Comma),k(Dot), k(Slash), k(Bslash)  ],
        [Trans,      Trans,k(LGui),k(LAlt),L1_SP,k(LCtrl),k(RShift),L2_ENTER,k(RAlt),k(BSpace),Trans,   Trans      ],
    ], [
        [k(F1),         k(F2),   k(F3),     k(F4),     k(F5),    k(F6),k(F7),      k(F8),  k(F9),    k(F10), k(F11),  k(F12)],
        [Trans,         k(Pause),Trans,     k(PScreen),Trans,    Trans,Trans,      Trans,  k(Delete),Trans,  Trans,   Trans ],
        [Trans,         Trans,   k(NumLock),k(Insert), k(Escape),Trans,k(CapsLock),k(Left),k(Down),  k(Up),  k(Right),Trans ],
        [k(NonUsBslash),k(Undo), CUT,       COPY,      PASTE,    Trans,Trans,      k(Home),k(PgDown),k(PgUp),k(End),  Trans ],
        [Trans,         Trans,   Trans,     Trans,     Trans,    Trans,Trans,      Trans,  Trans,    Trans,  Trans,   Trans ],
    ], [
        [Trans,    Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans    ],
        [s!(Grave),s!(Kb1),s!(Kb2),s!(Kb3),s!(Kb4),s!(Kb5),s!(Kb6),s!(Kb7),s!(Kb8),s!(Kb9),s!(Kb0),s!(Minus)],
        [ k(Grave), k(Kb1), k(Kb2), k(Kb3), k(Kb4), k(Kb5), k(Kb6), k(Kb7), k(Kb8), k(Kb9), k(Kb0), k(Minus)],
        [a!(Grave),a!(Kb1),a!(Kb2),a!(Kb3),a!(Kb4),a!(Kb5),a!(Kb6),a!(Kb7),a!(Kb8),a!(Kb9),a!(Kb0),a!(Minus)],
        [Trans,    Trans,  Trans,  Trans,  CSPACE, Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans    ],
    ], [
        [Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
        [k(F1),k(F2),k(F3),k(F4),k(F5),k(F6),k(F7),k(F8),k(F9),k(F10),k(F11),k(F12)],
        [Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
        [Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
        [Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
    ],
];

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        #[lock_free]
        layout: Layout<12, 5, 4>,
    }

    #[local]
    struct Local {
        matrix: Matrix<EPin<Input<PullUp>>, EPin<Output<PushPull>>, 12, 5>,
        debouncer: Debouncer<[[bool; 12]; 5]>,
        timer: timer::CounterHz<pac::TIM3>,
    }

    #[init(local = [bus: Option<UsbBusAllocator<usb::UsbBusType>> = None])]
    fn init(mut c: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut flash = c.device.FLASH.constrain();
        let rcc = c.device.RCC.constrain();

        // set 0x424C in DR10 for dfu on reset
        let bkp = rcc.bkp.constrain(c.device.BKP, &mut c.device.PWR);
        bkp.write_data_register_low(9, 0x424C);

        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        let mut gpioa = c.device.GPIOA.split();
        let mut gpiob = c.device.GPIOB.split();
        let mut gpioc = c.device.GPIOC.split();

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        cortex_m::asm::delay(clocks.sysclk().raw() / 100);

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        led.set_high();
        let leds = Leds { caps_lock: led };

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: c.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        *c.local.bus = Some(UsbBus::new(usb));
        let usb_bus = c.local.bus.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, leds);
        let usb_dev = keyberon::new_device(usb_bus);

        let mut timer = c.device.TIM3.counter_hz(&clocks);
        timer.start(1.kHz()).unwrap();
        timer.listen(timer::Event::Update);

        let matrix = Matrix::new(
            [
                gpiob.pb12.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb13.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb14.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb15.into_pull_up_input(&mut gpiob.crh).erase(),
                gpioa.pa8.into_pull_up_input(&mut gpioa.crh).erase(),
                gpioa.pa9.into_pull_up_input(&mut gpioa.crh).erase(),
                gpioa.pa10.into_pull_up_input(&mut gpioa.crh).erase(),
                gpiob.pb5.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb6.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb7.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb8.into_pull_up_input(&mut gpiob.crh).erase(),
                gpiob.pb9.into_pull_up_input(&mut gpiob.crh).erase(),
            ],
            [
                gpiob.pb11.into_push_pull_output(&mut gpiob.crh).erase(),
                gpiob.pb10.into_push_pull_output(&mut gpiob.crh).erase(),
                gpiob.pb1.into_push_pull_output(&mut gpiob.crl).erase(),
                gpiob.pb0.into_push_pull_output(&mut gpiob.crl).erase(),
                gpioa.pa7.into_push_pull_output(&mut gpioa.crl).erase(),
            ],
        );

        (
            Shared {
                usb_dev,
                usb_class,
                layout: Layout::new(&LAYERS),
            },
            Local {
                matrix: matrix.unwrap(),
                debouncer: Debouncer::new([[false; 12]; 5], [[false; 12]; 5], 5),
                timer,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = USB_HP_CAN_TX, priority = 2, shared = [usb_dev, usb_class])]
    fn usb_tx(c: usb_tx::Context) {
        (c.shared.usb_dev, c.shared.usb_class)
            .lock(|usb_dev, usb_class| usb_poll(usb_dev, usb_class))
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 2, shared = [usb_dev, usb_class])]
    fn usb_rx(c: usb_rx::Context) {
        (c.shared.usb_dev, c.shared.usb_class)
            .lock(|usb_dev, usb_class| usb_poll(usb_dev, usb_class))
    }

    #[task(binds = TIM3, priority = 1, local = [matrix, debouncer, timer], shared = [usb_class, layout])]
    fn tick(mut c: tick::Context) {
        c.local.timer.clear_interrupt(timer::Event::Update);

        for event in c.local.debouncer.events(c.local.matrix.get().unwrap()) {
            c.shared.layout.event(event);
        }
        c.shared.layout.tick();

        let report = c.shared.layout.keycodes();
        c.shared.usb_class.lock(|k| send_report(report, k));
    }
}

fn send_report(
    iter: impl Iterator<Item = KeyCode>,
    usb_class: &mut HidClass<'_, UsbBus<Peripheral>, Keyboard<Leds>>,
) {
    let report: KbHidReport = iter.collect();
    if usb_class.device_mut().set_keyboard_report(report.clone()) {
        while let Ok(0) = usb_class.write(report.as_bytes()) {}
    }
}

fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
