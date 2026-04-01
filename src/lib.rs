use tauri::command;

// 피치 시프트 (고양이 목소리로 변환)
fn pitch_shift(samples: &[i16], factor: f32) -> Vec<i16> {
    let new_len = (samples.len() as f32 / factor) as usize;
    let mut output = Vec::with_capacity(new_len);
    for i in 0..new_len {
        let src_idx = (i as f32 * factor) as usize;
        if src_idx < samples.len() {
            output.push(samples[src_idx]);
        }
    }
    output
}

// meow 효과 오버레이
fn add_meow_effect(samples: &mut Vec<i16>, intensity: f32) {
    let freq = 850.0_f32; // 고양이 울음 주파수
    for (i, sample) in samples.iter_mut().enumerate() {
        let t = i as f32 / 44100.0;
        let meow = (2.0 * std::f32::consts::PI * freq * t).sin();
        let mixed = (*sample as f32) * (1.0 - intensity * 0.3)
            + meow * i16::MAX as f32 * intensity * 0.3;
        *sample = mixed.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
    }
}

#[command]
fn process_audio(intensity: f32) -> String {
    // Mock: 테스트용 사인파 생성 (실제 마이크 대신)
    let sample_rate = 44100u32;
    let duration = 2.0_f32;
    let mut samples: Vec<i16> = (0..(sample_rate as f32 * duration) as usize)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let wave = (2.0 * std::f32::consts::PI * 200.0 * t).sin();
            (wave * i16::MAX as f32 * 0.5) as i16
        })
        .collect();

    // 피치 올리기 (고양이처럼)
    let shifted = pitch_shift(&samples, 0.5 + intensity * 0.3);
    samples = shifted;

    // meow 효과 추가
    add_meow_effect(&mut samples, intensity);

    // WAV 파일로 저장
    let path = "/tmp/meow_output.wav";
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for s in &samples {
        writer.write_sample(*s).unwrap();
    }
    writer.finalize().unwrap();

    format!("success:{}", path)
}

#[command]
fn get_status() -> String {
    "Meow Changer Ready 🐱".to_string()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![process_audio, get_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
