#![cfg_attr(feature = "bundle", windows_subsystem = "windows")]
use dioxus::prelude::*;
use std::time::Duration;

use windows::Win32::System::Com::*;
use windows::Win32::{Media::Audio::Endpoints::*, Media::Audio::*};

static GULP: Asset = asset!("/assets/gulp.webp");
static AQUARIUM: Asset = asset!("/assets/fish.mp4");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Hero {}
    }
}

#[component]
pub fn Hero() -> Element {
    let mut volume_thres: Signal<u32> = use_signal(|| 6);
    let mut volume_level: Signal<f32> = use_signal(|| 0.0);
    let current_volume = volume_level();
    let current_thres = volume_thres();

    let handle_record = use_callback(move |_| {
        spawn(async move {
            unsafe {
                let _ = CoInitialize(None);

                let mmdevice_enumerator: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
                let device_collection = mmdevice_enumerator
                    .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
                    .unwrap();

                // Activate second device (webcam). First device is Nvidia RTX voice
                let device = device_collection.Item(1).unwrap();
                let audio_meter_information: IAudioMeterInformation = device
                    .Activate::<IAudioMeterInformation>(CLSCTX_ALL, None)
                    .unwrap();

                loop {
                    let peak_value = audio_meter_information.GetPeakValue().unwrap();
                    volume_level.set(peak_value);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });
    });

    rsx! {
        document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css")
        },
        div {
            class: "z-10 absolute flex gap-4 items-center h-14 w-full p-2",
            button {
                onclick: move |e| {
                    handle_record(e)
                },
                class: "bg-gray-200 rounded-md h-full border-2 px-2 border-gray-300 cursor-pointer",
                "Activate"
            },
            input {
                class: "bg-gray-200 rounded-md h-full w-24 px-2 text-center",
                type: "number",
                value: volume_thres(),
                onchange: move |e| {
                    volume_thres.set(e.value().parse::<u32>().unwrap());
                }
            },
            div {
                {volume_level.read().to_string()}
            }
        }
        video {
            class: "fixed min-w-screen min-h-screen right-0 bottom-0 object-cover",
            autoplay: true,
            muted: true,
            loop: true,
            controls: true,
            source {
                src: AQUARIUM
            },
        },
        div {
            class: "shake fixed w-screen h-screen z-10 top-0 right-0 pointer-events-none",
            if current_volume >= (current_thres as f32 / 100.0) {
                img {
                    class: "h-full w-full object-cover",
                    src: GULP
                }
            }
        }
    }
}
