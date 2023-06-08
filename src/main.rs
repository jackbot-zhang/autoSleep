use chrono::{Local, NaiveTime, Duration};
use evdev::{Device, InputEvent, InputEventKind};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Result;
use tokio;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

#[tokio::main]
async fn main() -> Result<()> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::default()))
        .build("/var/log/autoSleep.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();

    let mut event_device = Device::open("/dev/input/event4")?;
    let mut m_event = event_device.into_event_stream().unwrap();
    let mut buf = VecDeque::with_capacity(2);
    let mut last_time = Local::now();
    loop {
        let e = m_event.next_event().await?;
        match e.kind() {
            InputEventKind::Key(key) if matches!(key.code(), 125 | 38) => {
                let now = Local::now();

                let code = key.code();

                buf.push_back(code);

                if buf.len() > 2 {
                    buf.pop_front().unwrap();
                }

                if code == 38 && *(buf.front().unwrap()) == 125 {
                    let local_time = now.time();
                    let six_pm = NaiveTime::from_hms_opt(17, 59, 0).unwrap();
                    if local_time >= six_pm {
                        let elapsed_time = now - last_time;

                        if elapsed_time < Duration::milliseconds(500) {
                            off();
                        }
                    }
                }
                last_time = now
            }
            _ => {}
        }
    }
}

fn off() {
    use std::process::Command;

    let res = Command::new("/usr/bin/sh")
        .arg("-c")
        .arg("/usr/bin/systemctl suspend")
        .output();

    match res {
        Ok(o) => {
            if !o.status.success() {
                log::error!("{:?}", o)
            }
        }
        Err(e) => {
            log::error!("{}", e.to_string())
        }
    }
}
