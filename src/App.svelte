<script lang="ts">
  import { onDestroy } from 'svelte';

  // ── State ──────────────────────────────────────────────
  let isLive = false;          // 실시간 마이크 변환 ON/OFF
  let isRecording = false;     // 녹음 중
  let intensity = 0.5;
  let statusText = "Idle";
  let chatInput = "";
  let chatHistory: { text: string; type: 'user' | 'cat' }[] = [];
  let chatEnd: HTMLDivElement;
  let isProcessing = false;
  let recordedUrl: string | null = null;

  // ── Audio Core ─────────────────────────────────────────
  let audioCtx: AudioContext | null = null;
  let micStream: MediaStream | null = null;
  let processorNode: ScriptProcessorNode | null = null;
  let mediaRecorder: MediaRecorder | null = null;
  let recordedChunks: BlobPart[] = [];

  function getCtx(): AudioContext {
    if (!audioCtx) audioCtx = new AudioContext();
    return audioCtx;
  }

  // ── Live Mode ──────────────────────────────────────────
  async function toggleLive() {
    if (isLive) {
      stopLive();
    } else {
      try {
        await startLive();
      } catch {
        statusText = "Mic access denied";
      }
    }
  }

  async function startLive() {
    const ctx = getCtx();
    micStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
    const source = ctx.createMediaStreamSource(micStream);

    // 피치 시프트 (ScriptProcessor)
    processorNode = ctx.createScriptProcessor(2048, 1, 1);
    const pitchFactor = 0.45 + intensity * 0.4;
    processorNode.onaudioprocess = (e) => {
      const input = e.inputBuffer.getChannelData(0);
      const output = e.outputBuffer.getChannelData(0);
      for (let i = 0; i < output.length; i++) {
        const srcIdx = Math.floor(i * pitchFactor) % input.length;
        output[i] = input[srcIdx] * 1.2;
      }
    };

    // 포르만트 필터
    const filter = ctx.createBiquadFilter();
    filter.type = 'bandpass';
    filter.frequency.value = 900 + intensity * 500;
    filter.Q.value = 1.5;

    source.connect(filter);
    filter.connect(processorNode);
    processorNode.connect(ctx.destination);

    isLive = true;
    statusText = "Live — meowing...";
  }

  function stopLive() {
    processorNode?.disconnect();
    processorNode = null;
    micStream?.getTracks().forEach(t => t.stop());
    micStream = null;
    isLive = false;
    statusText = "Idle";
  }

  // ── Record Mode ────────────────────────────────────────
  async function toggleRecord() {
    if (isRecording) {
      stopRecording();
    } else {
      try {
        await startRecording();
      } catch {
        statusText = "Mic access denied";
      }
    }
  }

  async function startRecording() {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
    recordedChunks = [];
    mediaRecorder = new MediaRecorder(stream);
    mediaRecorder.ondataavailable = (e) => {
      if (e.data.size > 0) recordedChunks.push(e.data);
    };
    mediaRecorder.onstop = async () => {
      stream.getTracks().forEach(t => t.stop());
      await processRecording();
    };
    mediaRecorder.start();
    isRecording = true;
    statusText = "Recording...";
  }

  function stopRecording() {
    mediaRecorder?.stop();
    isRecording = false;
    statusText = "Processing...";
  }

  async function processRecording() {
    isProcessing = true;
    try {
      const blob = new Blob(recordedChunks, { type: 'audio/webm' });
      const arrayBuffer = await blob.arrayBuffer();
      const ctx = getCtx();
      const decoded = await ctx.decodeAudioData(arrayBuffer);

      // 고양이 필터 적용
      const pitchFactor = 0.45 + intensity * 0.4;
      const outLength = Math.floor(decoded.length / pitchFactor);
      const offCtx = new OfflineAudioContext(1, outLength, decoded.sampleRate);

      const src = offCtx.createBufferSource();
      src.buffer = decoded;
      src.playbackRate.value = pitchFactor;

      const f1 = offCtx.createBiquadFilter();
      f1.type = 'bandpass';
      f1.frequency.value = 900 + intensity * 500;
      f1.Q.value = 1.5;

      const f2 = offCtx.createBiquadFilter();
      f2.type = 'peaking';
      f2.frequency.value = 2400;
      f2.gain.value = 6;
      f2.Q.value = 2;

      const gain = offCtx.createGain();
      gain.gain.value = 1.3;

      src.connect(f1);
      f1.connect(f2);
      f2.connect(gain);
      gain.connect(offCtx.destination);
      src.start(0);

      const rendered = await offCtx.startRendering();

      // WAV 변환 후 재생
      const wavBlob = audioBufferToWav(rendered);
      if (recordedUrl) URL.revokeObjectURL(recordedUrl);
      recordedUrl = URL.createObjectURL(wavBlob);

      // 자동 재생
      const audio = new Audio(recordedUrl);
      audio.play();

      statusText = "Playback ready ▶";
    } catch (e) {
      statusText = "Error: " + e;
    }
    isProcessing = false;
  }

  // ── Chat WAV ───────────────────────────────────────────
  async function sendChat() {
    if (!chatInput.trim() || isProcessing) return;
    const text = chatInput.trim();
    chatInput = "";
    chatHistory = [...chatHistory, { text, type: 'user' }];
    isProcessing = true;

    try {
      // Tauri 환경이면 Rust 백엔드 사용, 아니면 Web Audio fallback
      let wavBlob: Blob;
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const path = await invoke<string>('generate_chat_wav', { text, intensity });
        // Tauri에서 파일 읽기
        const { readFile } = await import('@tauri-apps/plugin-fs');
        const bytes = await readFile(path);
        wavBlob = new Blob([bytes], { type: 'audio/wav' });
      } catch {
        // 브라우저 fallback
        wavBlob = generateFallbackWav(text, intensity);
      }

      const url = URL.createObjectURL(wavBlob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `meow_${Date.now()}.wav`;
      a.click();
      URL.revokeObjectURL(url);

      const pool = ['meow', 'nya', 'mrrrow', 'nyan', 'mew', 'nyaa', 'purr'];
      const n = text.replace(/\s/g, '').length;
      const reply = Array.from({ length: Math.ceil(n / 2) }, () =>
        pool[Math.floor(Math.random() * pool.length)]
      ).join(' ');

      chatHistory = [...chatHistory, { text: reply, type: 'cat' }];
      setTimeout(() => chatEnd?.scrollIntoView({ behavior: 'smooth' }), 50);
    } catch (e) {
      chatHistory = [...chatHistory, { text: 'mrrow... (error)', type: 'cat' }];
    }
    isProcessing = false;
  }

  function generateFallbackWav(text: string, intensity: number): Blob {
    const sampleRate = 44100;
    const syllables = text.replace(/\s/g, '').length;
    const sliceDur = 0.18 + (1 - intensity) * 0.12;
    const silenceDur = 0.04;
    const sliceSamples = Math.floor(sliceDur * sampleRate);
    const silenceSamples = Math.floor(silenceDur * sampleRate);
    const total = syllables * (sliceSamples + silenceSamples);

    const buf = new ArrayBuffer(44 + total * 2);
    const v = new DataView(buf);
    const ws = (o: number, s: string) => { for (let i = 0; i < s.length; i++) v.setUint8(o + i, s.charCodeAt(i)); };
    ws(0,'RIFF'); v.setUint32(4,36+total*2,true); ws(8,'WAVE');
    ws(12,'fmt '); v.setUint32(16,16,true); v.setUint16(20,1,true);
    v.setUint16(22,1,true); v.setUint32(24,sampleRate,true); v.setUint32(28,sampleRate*2,true);
    v.setUint16(32,2,true); v.setUint16(34,16,true);
    ws(36,'data'); v.setUint32(40,total*2,true);

    let offset = 44;
    for (let i = 0; i < syllables; i++) {
      const pos = i / Math.max(syllables - 1, 1);
      const pitch = 500 + Math.sin(pos * Math.PI) * 400 * (0.5 + intensity);
      for (let j = 0; j < sliceSamples; j++) {
        const t = j / sampleRate;
        const env = j < sliceSamples * 0.1 ? j / (sliceSamples * 0.1) :
                    j > sliceSamples * 0.8 ? 1 - (j - sliceSamples * 0.8) / (sliceSamples * 0.2) : 1;
        const s = Math.sin(2 * Math.PI * pitch * t) * 0.6
                + Math.sin(2 * Math.PI * pitch * 2 * t) * 0.25
                + Math.sin(2 * Math.PI * pitch * 3 * t) * 0.1;
        v.setInt16(offset, Math.max(-32767, Math.min(32767, s * env * 32767)), true);
        offset += 2;
      }
      for (let j = 0; j < silenceSamples; j++) { v.setInt16(offset, 0, true); offset += 2; }
    }
    return new Blob([buf], { type: 'audio/wav' });
  }

  function audioBufferToWav(buffer: AudioBuffer): Blob {
    const n = buffer.length;
    const sr = buffer.sampleRate;
    const ab = new ArrayBuffer(44 + n * 2);
    const v = new DataView(ab);
    const ws = (o: number, s: string) => { for (let i = 0; i < s.length; i++) v.setUint8(o + i, s.charCodeAt(i)); };
    ws(0,'RIFF'); v.setUint32(4,36+n*2,true); ws(8,'WAVE');
    ws(12,'fmt '); v.setUint32(16,16,true); v.setUint16(20,1,true);
    v.setUint16(22,1,true); v.setUint32(24,sr,true); v.setUint32(28,sr*2,true);
    v.setUint16(32,2,true); v.setUint16(34,16,true);
    ws(36,'data'); v.setUint32(40,n*2,true);
    const d = buffer.getChannelData(0);
    for (let i = 0; i < n; i++) v.setInt16(44+i*2, Math.max(-1,Math.min(1,d[i]))*32767, true);
    return new Blob([ab], { type: 'audio/wav' });
  }

  function handleKey(e: KeyboardEvent) { if (e.key === 'Enter') sendChat(); }

  // intensity 바뀌면 live 재시작
  $: if (isLive && intensity !== undefined) {
    stopLive();
    startLive().catch(() => { isLive = false; });
  }

  onDestroy(() => { stopLive(); });
</script>

<main>
  <!-- 배경 발바닥 -->
  <div class="bg">
    <span class="paw p1">🐾</span>
    <span class="paw p2">🐾</span>
    <span class="paw p3">🐾</span>
    <span class="paw p4">🐾</span>
  </div>

  <div class="panel">

    <!-- Header -->
    <div class="header">
      <span class="logo">🐱 Meow Changer</span>
      <span class="status-badge" class:on={isLive}>{isLive ? "● LIVE" : "○ OFF"}</span>
    </div>

    <!-- Intensity -->
    <div class="row">
      <span class="label">Intensity</span>
      <input type="range" min="0" max="1" step="0.01" bind:value={intensity} />
      <span class="val">{Math.round(intensity * 100)}%</span>
    </div>

    <!-- Buttons -->
    <div class="btns">
      <!-- Live Toggle -->
      <button class="btn-live {isLive ? 'on' : ''}" on:click={toggleLive}>
        {isLive ? '■ Stop Live' : '▶ Start Live'}
      </button>

      <!-- Record Toggle -->
      <button
        class="btn-rec {isRecording ? 'on' : ''}"
        on:click={toggleRecord}
        disabled={isProcessing}
      >
        {isRecording ? '■ Stop' : '⏺ Record'}
      </button>

      <!-- Playback -->
      {#if recordedUrl}
        <button class="btn-play" on:click={() => new Audio(recordedUrl!).play()}>
          ▶ Play
        </button>
      {/if}
    </div>

    <!-- Status -->
    <div class="status-row">
      <span class="status-text">{isProcessing ? "Processing..." : statusText}</span>
    </div>

    <!-- Chat -->
    <div class="chat">
      <div class="messages">
        {#if chatHistory.length === 0}
          <div class="hint">Type to test cat WAV output ↓</div>
        {/if}
        {#each chatHistory as msg}
          <div class="msg {msg.type}">{msg.text}</div>
        {/each}
        <div bind:this={chatEnd}></div>
      </div>
      <div class="input-row">
        <input
          class="text-input"
          type="text"
          placeholder="Type something..."
          bind:value={chatInput}
          on:keydown={handleKey}
          disabled={isProcessing}
        />
        <button class="send" on:click={sendChat} disabled={isProcessing}>
          {isProcessing ? '...' : 'Meow!'}
        </button>
      </div>
    </div>

  </div>
</main>

<style>
  @import url('https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;600;700&display=swap');

  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    background: #0e0e16;
    font-family: 'DM Sans', sans-serif;
    width: 360px;
    height: 520px;
    overflow: hidden;
    user-select: none;
  }

  main {
    width: 360px;
    height: 520px;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* 배경 발바닥 */
  .bg { position: absolute; inset: 0; pointer-events: none; overflow: hidden; }
  .paw {
    position: absolute;
    font-size: 5rem;
    opacity: 0.05;
    animation: drift 7s ease-in-out infinite;
  }
  .p1 { top: 4%;  left: 2%;  font-size: 4.5rem; animation-delay: 0s; }
  .p2 { top: 48%; left: 68%; font-size: 7rem;   animation-delay: 2s; }
  .p3 { top: 76%; left: 8%;  font-size: 3.8rem; animation-delay: 4s; }
  .p4 { top: 14%; left: 56%; font-size: 3.2rem; animation-delay: 1s; }
  @keyframes drift {
    0%,100% { transform: translateY(0) rotate(-15deg); }
    50%      { transform: translateY(-10px) rotate(-5deg); }
  }

  /* Panel */
  .panel {
    position: relative;
    z-index: 1;
    width: 320px;
    background: #16162a;
    border: 1px solid #ff6b9d33;
    border-radius: 16px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 0 40px #ff6b9d15;
  }

  /* Header */
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .logo {
    font-size: 0.95rem;
    font-weight: 700;
    color: #ff6b9d;
  }
  .status-badge {
    font-size: 0.72rem;
    font-weight: 600;
    color: #555;
    letter-spacing: 0.5px;
  }
  .status-badge.on {
    color: #4ade80;
    text-shadow: 0 0 8px #4ade8066;
    animation: pulse 1.5s infinite;
  }
  @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.6} }

  /* Intensity */
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .label { font-size: 0.73rem; color: #666; width: 56px; flex-shrink: 0; }
  input[type="range"] { flex: 1; accent-color: #ff6b9d; cursor: pointer; }
  .val { font-size: 0.73rem; color: #ff6b9d; width: 30px; text-align: right; flex-shrink: 0; }

  /* Buttons */
  .btns {
    display: flex;
    gap: 6px;
  }
  .btn-live, .btn-rec, .btn-play {
    flex: 1;
    padding: 8px 0;
    border-radius: 8px;
    border: none;
    font-family: 'DM Sans', sans-serif;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }
  .btn-live {
    background: #1e1e36;
    border: 1px solid #ff6b9d55;
    color: #ff6b9d;
  }
  .btn-live.on {
    background: #ff6b9d;
    color: #fff;
    border-color: #ff6b9d;
    box-shadow: 0 0 16px #ff6b9d55;
  }
  .btn-rec {
    background: #1e1e36;
    border: 1px solid #f8716155;
    color: #f87161;
  }
  .btn-rec.on {
    background: #f87161;
    color: #fff;
    border-color: #f87161;
    box-shadow: 0 0 16px #f8716155;
    animation: recpulse 1s infinite;
  }
  @keyframes recpulse { 0%,100%{box-shadow:0 0 16px #f8716155} 50%{box-shadow:0 0 28px #f87161aa} }
  .btn-rec:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-play {
    background: #1e1e36;
    border: 1px solid #4ade8055;
    color: #4ade80;
    flex: 0 0 auto;
    padding: 8px 12px;
  }
  .btn-play:hover { background: #4ade8022; }

  /* Status */
  .status-row {
    background: #0e0e16;
    border-radius: 8px;
    padding: 6px 10px;
    border: 1px solid #ffffff0a;
  }
  .status-text { font-size: 0.75rem; color: #555; }

  /* Chat */
  .chat { display: flex; flex-direction: column; gap: 8px; }
  .messages {
    background: #0e0e16;
    border-radius: 10px;
    padding: 10px;
    height: 170px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 5px;
    scrollbar-width: thin;
    scrollbar-color: #ff6b9d22 transparent;
  }
  .hint { color: #333; font-size: 0.75rem; text-align: center; margin: auto; }
  .msg {
    font-size: 0.8rem;
    padding: 5px 9px;
    border-radius: 9px;
    max-width: 82%;
    line-height: 1.4;
    word-break: break-word;
  }
  .msg.user {
    background: #ff6b9d1a;
    color: #eee;
    border: 1px solid #ff6b9d33;
    align-self: flex-end;
  }
  .msg.cat {
    background: #1e1e36;
    color: #aaa;
    border: 1px solid #ffffff0a;
    align-self: flex-start;
  }
  .msg.cat::before { content: '🐱 '; }

  /* Input */
  .input-row { display: flex; gap: 6px; }
  .text-input {
    flex: 1;
    background: #0e0e16;
    border: 1px solid #ff6b9d22;
    border-radius: 8px;
    padding: 7px 10px;
    color: #fff;
    font-family: 'DM Sans', sans-serif;
    font-size: 0.8rem;
    outline: none;
    transition: border-color 0.15s;
  }
  .text-input:focus { border-color: #ff6b9d66; }
  .text-input::placeholder { color: #333; }
  .text-input:disabled { opacity: 0.4; }
  .send {
    background: #ff6b9d;
    border: none;
    border-radius: 8px;
    padding: 7px 13px;
    color: #fff;
    font-family: 'DM Sans', sans-serif;
    font-weight: 700;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.15s;
    flex-shrink: 0;
  }
  .send:hover:not(:disabled) { background: #ff85b0; transform: scale(1.03); }
  .send:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
