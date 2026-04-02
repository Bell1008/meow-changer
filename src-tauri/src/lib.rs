use tauri::command;
use std::f32::consts::PI;

// 고양이 WAV 생성 (텍스트 음절 수 기반)
fn generate_meow_samples(syllable_count: usize, intensity: f32) -> Vec<i16> {
    let sample_rate = 44100u32;
    let slice_dur = 0.18f32 + (1.0 - intensity) * 0.12;
    let silence_dur = 0.04f32;
    let slice_samples = (slice_dur * sample_rate as f32) as usize;
    let silence_samples = (silence_dur * sample_rate as f32) as usize;

    let mut output = Vec::new();

    for i in 0..syllable_count {
        let pos = i as f32 / syllable_count.max(1) as f32;
        // 한국어 억양 곡선: 올라갔다 내려옴
        let pitch_curve = 0.85 + (pos * PI).sin() * 0.35 * (0.5 + intensity);
        let jitter: f32 = (rand::random::<f32>() - 0.5) * 0.12;
        let pitch = (pitch_curve + jitter).clamp(0.5, 1.8);

        // 고양이 배음 구조
        let base_freq = 500.0 + pitch * 400.0 * (0.5 + intensity);

        for j in 0..slice_samples {
            let t = j as f32 / sample_rate as f32;

            // 엔벨로프 (음절 느낌)
            let envelope = if t < 0.02 {
                t / 0.02
            } else if t < slice_dur * 0.75 {
                1.0
            } else {
                1.0 - (t - slice_dur * 0.75) / (slice_dur * 0.25)
            }.clamp(0.0, 1.0);

            // 배음 합성
            let f1 = (2.0 * PI * base_freq * t).sin();
            let f2 = (2.0 * PI * base_freq * 2.0 * t).sin() * 0.4;
            let f3 = (2.0 * PI * base_freq * 3.0 * t).sin() * 0.2;
            // 비브라토
            let vibrato = 1.0 + 0.03 * (2.0 * PI * 6.0 * t).sin();
            // 포르만트
            let formant = (2.0 * PI * (1200.0 + intensity * 800.0) * t).sin() * 0.15 * intensity;

            let sample = ((f1 + f2 + f3) * vibrato + formant) * envelope * 0.6;
            let clamped = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            output.push(clamped);
        }

        // 음절 사이 무음
        output.extend(vec![0i16; silence_samples]);
    }

    output
}

#[command]
fn generate_chat_wav(text: String, intensity: f32) -> Result<String, String> {
    let syllable_count = text.chars().filter(|c| !c.is_whitespace()).count().max(1);
    let samples = generate_meow_samples(syllable_count, intensity);

    let path = std::env::temp_dir().join("meow_chat.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&path, spec)
        .map_err(|e| e.to_string())?;
    for s in &samples {
        writer.write_sample(*s).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[command]
fn get_status() -> String {
    "Meow Changer Ready".to_string()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            generate_chat_wav,
            get_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
