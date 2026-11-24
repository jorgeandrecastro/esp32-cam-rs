//! Serves a JPEG snapshot over HTTP on `/`.
//!
//! Fill in `WIFI_SSID` / `WIFI_PASS`, flash, then open the IP address printed
//! in the serial log.

use anyhow::Result;
use esp32_cam_rs::{Camera, Config, FrameSize};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration as WifiConfig, EspWifi};
use std::sync::{Arc, Mutex};

const WIFI_SSID: &str = "";
const WIFI_PASS: &str = "";

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    wifi.set_configuration(&WifiConfig::Client(ClientConfiguration {
        ssid: WIFI_SSID.try_into().unwrap(),
        password: WIFI_PASS.try_into().unwrap(),
        ..Default::default()
    }))?;
    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Connected! Open: http://{}/", ip_info.ip);

    let cam = Arc::new(Mutex::new(Camera::init_ai_thinker_with(&Config {
        frame_size: FrameSize::Svga,
        jpeg_quality: 12,
        ..Config::default()
    })?));

    let mut server = EspHttpServer::new(&HttpConfig::default())?;
    let cam_handler = Arc::clone(&cam);

    server.fn_handler("/", Method::Get, move |req| {
        let cam = cam_handler.lock().unwrap();
        let Some(frame) = cam.capture() else {
            let mut resp = req.into_status_response(503)?;
            resp.write_all(b"Camera capture failed")?;
            return Ok::<(), anyhow::Error>(());
        };

        if !frame.looks_like_jpeg() {
            let mut resp = req.into_status_response(503)?;
            resp.write_all(b"Incomplete JPEG frame (no SOI)")?;
            return Ok(());
        }

        let headers = [("Content-Type", "image/jpeg"), ("Cache-Control", "no-store")];
        let mut resp = req.into_response(200, Some("OK"), &headers)?;
        resp.write_all(&frame)?;
        Ok(())
    })?;

    core::mem::forget(server);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}