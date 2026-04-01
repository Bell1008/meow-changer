<script lang="ts">
  // Tauri 환경이 아닐 때 Mock으로 대체
  async function invokeCommand(cmd: string, args?: any): Promise<string> {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<string>(cmd, args);
    } catch {
      // 브라우저 Mock 모드
      if (cmd === "get_status") return "Meow Changer Ready 🐱 (브라우저 데모)";
      if (cmd === "process_audio") {
        await new Promise(r => setTimeout(r, 1500));
        return "success:/tmp/meow_output.wav";
      }
      return "mock_response";
    }
  }

  let intensity = 0.5;
  let status = "대기 중...";
  let isProcessing = false;
  let isActive = false;
  let resultPath = "";

  async function checkStatus() {
    status = await invokeCommand("get_status");
  }

  async function processAudio() {
    isProcessing = true;
    status = "🐱 냥냥 변환 중...";
    try {
      const res = await invokeCommand("process_audio", { intensity });
      if (res.startsWith("success:")) {
        resultPath = res.replace("success:", "");
        status = "✅ 변환 완료! WAV 파일 생성됨";
      }
    } catch (e) {
      status = "❌ 오류: " + e;
    }
    isProcessing = false;
  }

  function toggleActive() {
    isActive = !isActive;
    status = isActive ? "🔴 실시간 변환 ON (데모 모드)" : "⏸ 일시정지";
  }

  checkStatus();
</script>

<main>
  <div class="bg" aria-hidden="true">
    <div class="paw">🐾</div>
    <div class="paw paw2">🐾</div>
    <div class="paw paw3">🐾</div>
  </div>

  <div class="card">
    <div class="cat-face" class:active={isActive}>
      <span class="cat-emoji">{isActive ? "😸" : "😺"}</span>
    </div>

    <h1>Meow Changer</h1>
    <p class="subtitle">당신의 목소리를 고양이로 🐱</p>

    <div class="status-box">
      <span class="status-dot" class:on={isActive}></span>
      <span class="status-text">{status}</span>
    </div>

    <div class="slider-section">
      <label for="intensity">
        냥냥 강도
        <span class="intensity-label">
          {#if intensity < 0.3}살짝 야옹{:else if intensity < 0.6}보통 냥냥{:else if intensity < 0.85}완전 냥{:else}🐱 MAX 냥!{/if}
        </span>
      </label>
      <input
        id="intensity"
        type="range"
        min="0"
        max="1"
        step="0.01"
        bind:value={intensity}
      />
      <div class="slider-track-label">
        <span>약하게</span>
        <span>강하게</span>
      </div>
    </div>

    <div class="presets">
      <button class="preset" on:click={() => (intensity = 0.2)}>🐱 살살</button>
      <button class="preset" on:click={() => (intensity = 0.5)}>😸 보통</button>
      <button class="preset" on:click={() => (intensity = 0.8)}>🙀 강하게</button>
      <button class="preset" on:click={() => (intensity = 1.0)}>👹 MAX</button>
    </div>

    <div class="buttons">
      <button class="btn-test" on:click={processAudio} disabled={isProcessing}>
        {isProcessing ? "🔄 변환 중..." : "🎵 테스트 WAV 생성"}
      </button>
      <button class="btn-toggle" class:on={isActive} on:click={toggleActive}>
        {isActive ? "⏹ 중지" : "▶ 실시간 시작"}
      </button>
    </div>

    {#if resultPath}
      <div class="result-box">
        <p>📁 저장 위치:</p>
        <code>{resultPath}</code>
        <p class="hint">터미널에서 확인 후 다운로드하세요</p>
      </div>
    {/if}

    <div class="footer">
      <span>⌨️ 단축키: <kbd>Ctrl+M</kbd> 토글</span>
      <span>가상 오디오 케이블 필요 (실제 사용 시)</span>
    </div>
  </div>
</main>

<style>
  @import url('https://fonts.googleapis.com/css2?family=Nunito:wght@400;700;900&display=swap');

  :global(body) {
    margin: 0;
    background: #0f0f1a;
    font-family: 'Nunito', sans-serif;
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  main {
    position: relative;
    width: 100%;
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .bg {
    position: fixed;
    inset: 0;
    pointer-events: none;
    overflow: hidden;
  }

  .paw {
    position: absolute;
    font-size: 8rem;
    opacity: 0.04;
    top: 10%;
    left: 5%;
    transform: rotate(-20deg);
    animation: float 6s ease-in-out infinite;
  }
  .paw2 {
    top: 60%;
    left: 75%;
    font-size: 12rem;
    animation-delay: 2s;
    animation-duration: 8s;
  }
  .paw3 {
    top: 80%;
    left: 20%;
    font-size: 6rem;
    animation-delay: 4s;
  }

  @keyframes float {
    0%, 100% { transform: translateY(0) rotate(-20deg); }
    50% { transform: translateY(-20px) rotate(-10deg); }
  }

  .card {
    background: #1a1a2e;
    border: 1px solid #ff6b9d44;
    border-radius: 24px;
    padding: 2.5rem 2rem;
    width: 100%;
    max-width: 420px;
    box-shadow: 0 0 60px #ff6b9d22, 0 20px 60px #00000066;
    text-align: center;
    position: relative;
    z-index: 1;
  }

  .cat-face {
    font-size: 5rem;
    margin-bottom: 0.5rem;
    transition: transform 0.3s;
  }
  .cat-face.active {
    animation: bounce 0.6s infinite alternate;
  }
  @keyframes bounce {
    from { transform: scale(1); }
    to { transform: scale(1.15) rotate(5deg); }
  }

  h1 {
    font-size: 2.2rem;
    font-weight: 900;
    color: #ff6b9d;
    margin: 0 0 0.2rem;
    letter-spacing: -1px;
    text-shadow: 0 0 20px #ff6b9d88;
  }

  .subtitle {
    color: #aaa;
    font-size: 0.95rem;
    margin: 0 0 1.5rem;
  }

  .status-box {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    background: #0f0f1a;
    border-radius: 12px;
    padding: 0.6rem 1rem;
    margin-bottom: 1.5rem;
    border: 1px solid #ffffff11;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #444;
    flex-shrink: 0;
    transition: background 0.3s;
  }
  .status-dot.on {
    background: #4ade80;
    box-shadow: 0 0 8px #4ade80;
    animation: pulse 1s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .status-text {
    color: #ccc;
    font-size: 0.85rem;
    text-align: left;
  }

  .slider-section {
    margin-bottom: 1rem;
  }

  label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: #ddd;
    font-size: 0.9rem;
    font-weight: 700;
    margin-bottom: 0.5rem;
  }

  .intensity-label {
    color: #ff6b9d;
    font-size: 0.85rem;
  }

  input[type="range"] {
    width: 100%;
    accent-color: #ff6b9d;
    cursor: pointer;
  }

  .slider-track-label {
    display: flex;
    justify-content: space-between;
    color: #666;
    font-size: 0.75rem;
    margin-top: 0.2rem;
  }

  .presets {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
  }

  .preset {
    background: #0f0f1a;
    border: 1px solid #ff6b9d44;
    color: #ddd;
    border-radius: 20px;
    padding: 0.3rem 0.8rem;
    font-size: 0.8rem;
    font-family: 'Nunito', sans-serif;
    cursor: pointer;
    transition: all 0.2s;
  }
  .preset:hover {
    border-color: #ff6b9d;
    color: #ff6b9d;
    transform: scale(1.05);
  }

  .buttons {
    display: flex;
    gap: 0.8rem;
    margin-bottom: 1rem;
  }

  .btn-test, .btn-toggle {
    flex: 1;
    padding: 0.8rem;
    border-radius: 14px;
    border: none;
    font-family: 'Nunito', sans-serif;
    font-weight: 700;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-test {
    background: #1e1e3a;
    border: 1px solid #ff6b9d66;
    color: #ff6b9d;
  }
  .btn-test:hover:not(:disabled) {
    background: #ff6b9d22;
    transform: translateY(-2px);
  }
  .btn-test:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-toggle {
    background: #2a2a2a;
    color: #aaa;
    border: 1px solid #444;
  }
  .btn-toggle.on {
    background: #ff6b9d;
    color: white;
    border-color: #ff6b9d;
    box-shadow: 0 0 20px #ff6b9d66;
  }
  .btn-toggle:hover {
    transform: translateY(-2px);
  }

  .result-box {
    background: #0f0f1a;
    border: 1px solid #4ade8044;
    border-radius: 12px;
    padding: 0.8rem;
    margin-bottom: 1rem;
    text-align: left;
  }
  .result-box p {
    color: #4ade80;
    font-size: 0.8rem;
    margin: 0 0 0.3rem;
  }
  .result-box code {
    color: #aaa;
    font-size: 0.75rem;
    word-break: break-all;
  }
  .hint {
    color: #666 !important;
    font-size: 0.72rem !important;
    margin-top: 0.3rem !important;
  }

  .footer {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    color: #555;
    font-size: 0.75rem;
    margin-top: 0.5rem;
    border-top: 1px solid #ffffff11;
    padding-top: 1rem;
  }

  kbd {
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 4px;
    padding: 0.1rem 0.3rem;
    font-size: 0.7rem;
    color: #aaa;
  }
</style>
