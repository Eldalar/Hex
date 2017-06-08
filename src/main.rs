extern crate alto;

use std::cmp::max;
use std::collections::VecDeque;
use alto::*;


fn main() {
    let alto = if let Ok(alto) = Alto::load_default() {
        alto
    } else {
        println!("No OpenAL implementation present!");
        return;
    };

    println!("Using output: {:?}", alto.default_output().unwrap());
    let dev = alto.open(None).unwrap();
    let ctx = dev.new_context(None).unwrap();
    let mut dev_cap : alto::Capture<Mono<i16>> = alto.open_capture(None, 44100, 1024).unwrap();

    let mut slot = if dev.is_extension_present(alto::ext::Alc::Efx) {
        println!("Using EFX reverb");
        let mut slot = ctx.new_aux_effect_slot().unwrap();
        let mut reverb: EaxReverbEffect = ctx.new_effect().unwrap();
        reverb.set_preset(&alto::REVERB_PRESET_GENERIC).unwrap();
        slot.set_effect(&reverb).unwrap();
        Some(slot)
    } else {
        println!("EFX not present");
        None
    };

    println!("Capturing...");

    {
        dev_cap.start();

        let mut src = ctx.new_streaming_source().unwrap();
        if let Some(ref mut slot) = slot {
            src.set_aux_send(0, slot).unwrap();
        }

        let mut buffer_queue : VecDeque<alto::Buffer> = VecDeque::<alto::Buffer>::new();
        for _ in 0 .. 5 {
            let mut buffer : Vec<Mono<i16>> = vec![];
            buffer.resize(2048 as usize, Mono::<i16> { center : 0 });
            let buf = ctx.new_buffer(buffer, 44_000).unwrap();
            buffer_queue.push_back(buf);
        }

        loop {
            let mut buffers_avail = src.buffers_processed();
            while buffers_avail > 0 {
                let buf = src.unqueue_buffer().unwrap();
                buffer_queue.push_back( buf );
                buffers_avail = buffers_avail - 1;
            }

            let samples_len = dev_cap.samples_len();
            if samples_len > 1024 { 
                let mut buffer : Vec<Mono<i16>> = vec![];
                buffer.resize(1024 as usize, Mono::<i16> { center : 0 });
                dev_cap.capture_samples( &mut buffer ).unwrap();

                if buffer_queue.len() > 0 {
                    let mut buf = buffer_queue.pop_front().unwrap();
                    for mut x in &mut buffer {
                        let mult = 5;
                        if x.center > (i16::max_value() / mult) {
                            x.center = i16::max_value();
                        } else if x.center < -(i16::max_value() / mult) {
                            x.center = -i16::max_value();
                        } else {
                            x.center = x.center * mult;
                        }
                    }
                    buf.set_data(buffer, 44000).unwrap();
                    //buf.set_data(wave.render().take(44_000 / 10).collect::<Vec<_>>(), 44_000).unwrap();
                    src.queue_buffer(buf);
                    if src.state() != alto::SourceState::Playing {
                        src.play()
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis( 8 ));
        }
        dev_cap.stop();
    }

    std::thread::sleep(std::time::Duration::new(1, 0));
}

