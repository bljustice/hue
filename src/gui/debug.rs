use crate::config;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;
use vizia_plug::vizia::prelude::*;

pub struct DebugContainer;

impl View for DebugContainer {
    fn element(&self) -> Option<&'static str> {
        Some("debug-container")
    }
}

impl DebugContainer {
    pub fn new(cx: &mut Context, debug: config::Debug) -> Handle<Self> {
        let frame = SyncSignal::new(0u32);
        let tick = frame.clone();
        let timer = cx.add_timer(Duration::from_millis(50), None, move |_cx, action| {
            if matches!(action, TimerAction::Tick(_)) {
                tick.update(|count| *count += 1);
            }
        });
        cx.start_timer(timer);

        Self.build(cx, |cx| {
            VStack::new(cx, |cx| {
                debug_line(cx, &frame, &debug, "Current sample value", |d| {
                    d.current_sample_val.load(Relaxed)
                });
                debug_line(cx, &frame, &debug, "Min sample value seen", |d| {
                    d.min_sample_val.load(Relaxed)
                });
                debug_line(cx, &frame, &debug, "Max sample value seen", |d| {
                    d.max_sample_val.load(Relaxed)
                });
                debug_line(cx, &frame, &debug, "Current sampling rate", |d| {
                    d.sample_rate.load(Relaxed)
                });
                debug_line(cx, &frame, &debug, "Output buffer len", |d| {
                    d.output_buffer.load(Relaxed)
                });
                debug_line(cx, &frame, &debug, "Mix level", |d| d.mix.load(Relaxed));
                debug_line(cx, &frame, &debug, "Gain level", |d| d.gain.load(Relaxed));
                debug_line(cx, &frame, &debug, "Envelope", |d| d.envelope.load(Relaxed));
            })
            .class("debug-container")
            .background_color(Color::rgb(255, 255, 255))
            .color(Color::rgb(0x69, 0x69, 0x69));
        })
    }
}

fn debug_line(
    cx: &mut Context,
    frame: &SyncSignal<u32>,
    debug: &config::Debug,
    label: &'static str,
    value: impl Fn(&config::Debug) -> f32 + 'static,
) {
    let frame = frame.clone();
    let debug = debug.clone();
    let text = Memo::new(move |_| {
        let _ = frame.get();
        format!("{label}: {}", value(&debug))
    });
    HStack::new(cx, |cx| {
        Label::new(cx, text);
    });
}
