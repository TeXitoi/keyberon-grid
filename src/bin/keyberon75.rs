#![no_main]
#![no_std]

use embedded_hal::digital::v2::OutputPin;
use keyberon::action::Action::{self, *};
use keyberon::action::{d, k, l, m, HoldTapConfig};
use keyberon::debounce::Debouncer;
use keyberon::key_code::KeyCode::*;
use keyberon::key_code::{KbHidReport, KeyCode};
use keyberon::layout::Layout;
use keyberon::matrix::{Matrix, PressedKeys};
use panic_halt as _;
use rtic::app;
use stm32f1xx_hal::gpio::{Input, Output, PullUp, PushPull, Pxx};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32f1xx_hal::{gpio, pac, timer};
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
            self.caps_lock.set_low().unwrap()
        } else {
            self.caps_lock.set_high().unwrap()
        }
    }
}

const CUT: Action = m(&[LShift, Delete]);
const COPY: Action = m(&[LCtrl, Insert]);
const PASTE: Action = m(&[LShift, Insert]);
const C_ENTER: Action = HoldTap {
    timeout: 200,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    tap_hold_interval: 0,
    hold: &k(LCtrl),
    tap: &k(Enter),
};
const L1_SP: Action = HoldTap {
    timeout: 200,
    config: HoldTapConfig::Default,
    tap_hold_interval: 0,
    hold: &l(1),
    tap: &k(Space),
};
const CENTER: Action = m(&[LCtrl, Enter]);

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers = &[
    &[
        &[k(Grave),   k(Kb1),k(Kb2),k(Kb3), k(Kb4),k(Kb5),k(KpMinus),k(KpSlash),k(KpAsterisk),k(Kb6),   k(Kb7), k(Kb8), k(Kb9),  k(Kb0),   k(Minus)   ],
        &[k(Tab),     k(Q),  k(W),  k(E),   k(R),  k(T),     k(Kp7), k(Kp8),    k(Kp9),       k(Y),     k(U),   k(I),   k(O),    k(P),     k(LBracket)],
        &[k(RBracket),k(A),  k(S),  k(D),   k(F),  k(G),     k(Kp4), k(Kp5),    k(Kp6),       k(H),     k(J),   k(K),   k(L),    k(SColon),k(Quote)   ],
        &[k(Equal),   k(Z),  k(X),  k(C),   k(V),  k(B),     k(Kp1), k(Kp2),    k(Kp3),       k(N),     k(M),   k(Comma),k(Dot), k(Slash), k(Bslash)  ],
        &[Trans,      Trans, k(LGui),k(LAlt),L1_SP,k(LShift),k(Kp0), k(KpDot),  k(KpPlus),    k(RShift),C_ENTER,k(RAlt),k(BSpace),Trans,   Trans      ],
    ], &[
        &[k(F1),k(F2),k(F3),     k(F4),k(F5),    k(F6),Trans,Trans,Trans,k(F7),      k(F8),  k(F9),    k(F10), k(F11),  k(F12)],
        &[Trans,Trans,Trans,     Trans,Trans,    Trans,Trans,Trans,Trans,Trans,      Trans,  k(Delete),Trans,  Trans,   Trans ],
        &[d(0), d(1), k(NumLock),Trans,k(Escape),Trans,Trans,Trans,Trans,k(CapsLock),k(Left),k(Down),  k(Up),  k(Right),Trans ],
        &[Trans,Trans,CUT,       COPY, PASTE,    Trans,Trans,Trans,Trans,Trans,      k(Home),k(PgDown),k(PgUp),k(End),  Trans ],
        &[Trans,Trans,Trans,     Trans,Trans,    Trans,Trans,Trans,Trans,Trans,      CENTER, Trans,    Trans,  Trans,   Trans ],
    ],
];

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        matrix: Matrix<Pxx<Input<PullUp>>, Pxx<Output<PushPull>>, 15, 5>,
        debouncer: Debouncer<PressedKeys<15, 5>>,
        layout: Layout,
        timer: timer::CountDownTimer<pac::TIM3>,
    }

    #[init]
    fn init(mut c: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

        let mut flash = c.device.FLASH.constrain();
        let mut rcc = c.device.RCC.constrain();

        // set 0x424C in DR10 for dfu on reset
        let bkp = rcc
            .bkp
            .constrain(c.device.BKP, &mut rcc.apb1, &mut c.device.PWR);
        bkp.write_data_register_low(9, 0x424C);

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        let mut gpioa = c.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = c.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = c.device.GPIOC.split(&mut rcc.apb2);

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low().unwrap();
        cortex_m::asm::delay(clocks.sysclk().0 / 100);

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        led.set_high().unwrap();
        let leds = Leds { caps_lock: led };

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: c.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        *USB_BUS = Some(UsbBus::new(usb));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, leds);
        let usb_dev = keyberon::new_device(usb_bus);

        let mut timer =
            timer::Timer::tim3(c.device.TIM3, &clocks, &mut rcc.apb1).start_count_down(1.khz());
        timer.listen(timer::Event::Update);

        let matrix = Matrix::new(
            [
                gpiob.pb12.into_pull_up_input(&mut gpiob.crh).downgrade(),
                gpiob.pb13.into_pull_up_input(&mut gpiob.crh).downgrade(),
                gpiob.pb14.into_pull_up_input(&mut gpiob.crh).downgrade(),
                gpiob.pb15.into_pull_up_input(&mut gpiob.crh).downgrade(),
                gpioa.pa8.into_pull_up_input(&mut gpioa.crh).downgrade(),
                gpioa.pa9.into_pull_up_input(&mut gpioa.crh).downgrade(),
                gpioa.pa10.into_pull_up_input(&mut gpioa.crh).downgrade(),
                gpiob.pb5.into_pull_up_input(&mut gpiob.crl).downgrade(),
                gpiob.pb6.into_pull_up_input(&mut gpiob.crl).downgrade(),
                gpiob.pb7.into_pull_up_input(&mut gpiob.crl).downgrade(),
                gpiob.pb8.into_pull_up_input(&mut gpiob.crh).downgrade(),
                gpiob.pb9.into_pull_up_input(&mut gpiob.crh).downgrade(),
                gpioa.pa6.into_pull_up_input(&mut gpioa.crl).downgrade(),
                gpioa.pa5.into_pull_up_input(&mut gpioa.crl).downgrade(),
                gpioa.pa4.into_pull_up_input(&mut gpioa.crl).downgrade(),
            ],
            [
                gpiob.pb11.into_push_pull_output(&mut gpiob.crh).downgrade(),
                gpiob.pb10.into_push_pull_output(&mut gpiob.crh).downgrade(),
                gpiob.pb1.into_push_pull_output(&mut gpiob.crl).downgrade(),
                gpiob.pb0.into_push_pull_output(&mut gpiob.crl).downgrade(),
                gpioa.pa7.into_push_pull_output(&mut gpioa.crl).downgrade(),
            ],
        );

        init::LateResources {
            usb_dev,
            usb_class,
            timer,
            debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
            matrix: matrix.unwrap(),
            layout: Layout::new(LAYERS),
        }
    }

    #[task(binds = USB_HP_CAN_TX, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_tx(mut c: usb_tx::Context) {
        usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_rx(mut c: usb_rx::Context) {
        usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
    }

    #[task(binds = TIM3, priority = 1, resources = [usb_class, matrix, debouncer, layout, timer])]
    fn tick(mut c: tick::Context) {
        c.resources.timer.clear_update_interrupt_flag();

        for event in c
            .resources
            .debouncer
            .events(c.resources.matrix.get().unwrap())
        {
            c.resources.layout.event(event);
        }
        c.resources.layout.tick();
        send_report(c.resources.layout.keycodes(), &mut c.resources.usb_class);
    }
};

fn send_report(iter: impl Iterator<Item = KeyCode>, usb_class: &mut resources::usb_class<'_>) {
    use rtic::Mutex;
    let report: KbHidReport = iter.collect();
    if usb_class.lock(|k| k.device_mut().set_keyboard_report(report.clone())) {
        while let Ok(0) = usb_class.lock(|k| k.write(report.as_bytes())) {}
    }
}

fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
