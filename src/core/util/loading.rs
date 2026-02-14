use core::f32::consts::PI;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

/// ローディングスピナーの状態
static SPINNER_RUNNING: AtomicBool = AtomicBool::new(false);
static SPINNER_STATE: Mutex<Option<SpinnerState>> = Mutex::new(None);
static SCREEN_CLEARED: AtomicBool = AtomicBool::new(false);
static FRAME_COUNTER: AtomicU64 = AtomicU64::new(0);

struct SpinnerState {
    framebuffer: u64,
    width: usize,
    height: usize,
    stride: usize,
    angle: f32,
    prev_dots: [(i32, i32); 8], // 前フレームのドット位置
}

/// ローディングスピナーを開始
pub fn start_loading(framebuffer: u64, width: usize, height: usize, stride: usize) {
    // デバッグ情報を出力
    crate::debug!("Loading spinner: width={}, height={}, stride={}", width, height, stride);

    let mut state = SPINNER_STATE.lock();
    *state = Some(SpinnerState {
        framebuffer,
        width,
        height,
        stride,
        angle: 0.0,
        prev_dots: [(0, 0); 8],
    });
    SPINNER_RUNNING.store(true, Ordering::SeqCst);
    SCREEN_CLEARED.store(false, Ordering::SeqCst);
    FRAME_COUNTER.store(0, Ordering::SeqCst);
}

/// ローディングスピナーを停止
pub fn end_kernel_load() {
    SPINNER_RUNNING.store(false, Ordering::SeqCst);
    let mut state = SPINNER_STATE.lock();
    *state = None;
}

/// ローディングスピナーが動作中かチェック
pub fn is_loading() -> bool {
    SPINNER_RUNNING.load(Ordering::SeqCst)
}

/// スピナーを1フレーム描画（タイマー割り込みから呼ばれる）
pub fn update_spinner() {
    if !is_loading() {
        return;
    }

    // フレームレート制限: 5フレームに1回だけ描画（10ms * 5 = 50ms = 20FPS）
    let frame = FRAME_COUNTER.fetch_add(1, Ordering::Relaxed);
    if frame % 5 != 0 {
        return;
    }

    let mut state = SPINNER_STATE.lock();
    if let Some(ref mut spinner) = *state {
        unsafe {
            draw_spinner(spinner);
        }
        spinner.angle += 0.15;
        if spinner.angle >= 2.0 * PI {
            spinner.angle -= 2.0 * PI;
        }
    }
}

/// ピクセルを設定（BGRフォーマット）
unsafe fn put_pixel(framebuffer: u64, stride: usize, x: usize, y: usize, r: u8, g: u8, b: u8, width: usize, height: usize) {
    if x >= width || y >= height {
        return;
    }
    // strideはすでにバイト単位（1ライン分のバイト数）
    let offset = y * stride + x * 4;
    let pixel = (framebuffer + offset as u64) as *mut u8;
    *pixel.add(0) = b; // Blue
    *pixel.add(1) = g; // Green
    *pixel.add(2) = r; // Red
    *pixel.add(3) = 0; // Reserved
}

/// 画面をクリア
unsafe fn clear_screen(state: &SpinnerState, r: u8, g: u8, b: u8) {
    for y in 0..state.height {
        for x in 0..state.width {
            put_pixel(state.framebuffer, state.stride, x, y, r, g, b, state.width, state.height);
        }
    }
}

/// ドットを描画（円形）
unsafe fn draw_dot(state: &SpinnerState, cx: i32, cy: i32, dot_radius: i32, r: u8, g: u8, b: u8) {
    for dy in -dot_radius..=dot_radius {
        for dx in -dot_radius..=dot_radius {
            if dx * dx + dy * dy <= dot_radius * dot_radius {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && x < state.width as i32 && y >= 0 && y < state.height as i32 {
                    put_pixel(state.framebuffer, state.stride, x as usize, y as usize, r, g, b, state.width, state.height);
                }
            }
        }
    }
}

/// Windowsスタイルのローディングスピナーを描画
unsafe fn draw_spinner(state: &mut SpinnerState) {
    let cx = (state.width / 2) as i32;
    let cy = (state.height / 2) as i32;
    let radius = 40;
    let dot_count = 8;
    let dot_radius = 5;

    // 初回のみ画面全体をクリア
    if !SCREEN_CLEARED.load(Ordering::SeqCst) {
        clear_screen(state, 0, 0, 0);
        SCREEN_CLEARED.store(true, Ordering::SeqCst);
    } else {
        // 前フレームのドットを消去（黒で塗りつぶす）
        for &(prev_x, prev_y) in &state.prev_dots {
            if prev_x != 0 || prev_y != 0 {
                draw_dot(state, prev_x, prev_y, dot_radius, 0, 0, 0);
            }
        }
    }

    // 新しいドットの位置を計算して描画
    let mut new_dots = [(0i32, 0i32); 8];
    for i in 0..dot_count {
        let dot_angle = state.angle + (i as f32 * 2.0 * PI / dot_count as f32);
        // 真円を描くために、X座標とY座標を同じスケールで計算
        let x = cx + (radius as f32 * libm::cosf(dot_angle)) as i32;
        let y = cy + (radius as f32 * libm::sinf(dot_angle)) as i32;

        // フェードアウト効果（先頭が明るく、後ろが暗くなる）
        let brightness = (255.0 * (1.0 - i as f32 / dot_count as f32)) as u8;

        draw_dot(state, x, y, dot_radius, brightness, brightness, brightness);
        new_dots[i] = (x, y);
    }

    // 現在のドット位置を保存
    state.prev_dots = new_dots;
}

