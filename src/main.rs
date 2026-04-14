use swiftlib::{
    ipc::{ipc_recv, ipc_send},
    keyboard::{read_scancode, read_scancode_tap},
    task::{find_process_by_name, yield_now},
};

const IPC_BUF_SIZE: usize = 4128;
const KAGAMI_PROCESS_CANDIDATES: [&str; 3] =
    ["/Applications/Kagami.app/entry.elf", "Kagami.app", "entry.elf"];

const OP_REQ_CREATE_WINDOW: u32 = 1;
const OP_RES_WINDOW_CREATED: u32 = 2;
const OP_REQ_FLUSH_CHUNK: u32 = 4;
const LAYER_APP: u8 = 3;

fn main() {
    println!("[Terminal] start");
    let kagami_tid = match parse_kagami_tid_from_args().or_else(find_kagami_tid) {
        Some(tid) => tid,
        None => {
            eprintln!("[Terminal] Kagami not found");
            return;
        }
    };

    let width: u16 = 720;
    let height: u16 = 420;
    let window_id = match create_window(kagami_tid, width, height) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[Terminal] create window failed: {}", e);
            return;
        }
    };

    let pixels = render_terminal_bootstrap(width as usize, height as usize);
    if let Err(e) = flush_window_chunked(kagami_tid, window_id, width, height, &pixels) {
        eprintln!("[Terminal] draw failed: {}", e);
        return;
    }
    println!("[Terminal] window shown");

    loop {
        let sc_opt = match read_scancode_tap() {
            Ok(Some(sc)) => Some(sc),
            Ok(None) => read_scancode(),
            Err(_) => read_scancode(),
        };
        if let Some(sc) = sc_opt
            && (sc == 0x01 || sc == 0x81)
        {
            println!("[Terminal] exit");
            return;
        }
        yield_now();
    }
}

fn create_window(kagami_tid: u64, width: u16, height: u16) -> Result<u32, &'static str> {
    let mut req = [0u8; 9];
    req[0..4].copy_from_slice(&OP_REQ_CREATE_WINDOW.to_le_bytes());
    req[4..6].copy_from_slice(&width.to_le_bytes());
    req[6..8].copy_from_slice(&height.to_le_bytes());
    req[8] = LAYER_APP;
    if (ipc_send(kagami_tid, &req) as i64) < 0 {
        return Err("send create window failed");
    }

    let mut recv = [0u8; IPC_BUF_SIZE];
    for _ in 0..256 {
        let (sender, len) = ipc_recv(&mut recv);
        if sender != kagami_tid || len < 8 {
            yield_now();
            continue;
        }
        let op = u32::from_le_bytes([recv[0], recv[1], recv[2], recv[3]]);
        if op != OP_RES_WINDOW_CREATED {
            continue;
        }
        return Ok(u32::from_le_bytes([recv[4], recv[5], recv[6], recv[7]]));
    }
    Err("window create timeout")
}

fn flush_window_chunked(
    kagami_tid: u64,
    window_id: u32,
    width: u16,
    height: u16,
    pixels: &[u32],
) -> Result<(), &'static str> {
    let total = width as usize * height as usize;
    if pixels.len() < total {
        return Err("pixel buffer too small");
    }
    let chunk_header = 20usize;
    let max_chunk_pixels = (IPC_BUF_SIZE - chunk_header) / 4;
    let width_usize = width as usize;
    let height_usize = height as usize;
    let chunk_w = width_usize.min(96).max(1);
    let chunk_h = (max_chunk_pixels / chunk_w).max(1);

    let mut y0 = 0usize;
    while y0 < height_usize {
        let h = (height_usize - y0).min(chunk_h);
        let mut x0 = 0usize;
        while x0 < width_usize {
            let w = (width_usize - x0).min(chunk_w);
            let mut msg = vec![0u8; chunk_header + (w * h * 4)];
            msg[0..4].copy_from_slice(&OP_REQ_FLUSH_CHUNK.to_le_bytes());
            msg[4..8].copy_from_slice(&window_id.to_le_bytes());
            msg[8..10].copy_from_slice(&width.to_le_bytes());
            msg[10..12].copy_from_slice(&height.to_le_bytes());
            msg[12..14].copy_from_slice(&(x0 as u16).to_le_bytes());
            msg[14..16].copy_from_slice(&(y0 as u16).to_le_bytes());
            msg[16..18].copy_from_slice(&(w as u16).to_le_bytes());
            msg[18..20].copy_from_slice(&(h as u16).to_le_bytes());
            let mut off = chunk_header;
            for row in 0..h {
                let src_row = (y0 + row) * width_usize;
                for col in 0..w {
                    msg[off..off + 4]
                        .copy_from_slice(&(pixels[src_row + x0 + col] | 0xFF00_0000).to_le_bytes());
                    off += 4;
                }
            }
            if (ipc_send(kagami_tid, &msg) as i64) < 0 {
                return Err("send flush chunk failed");
            }
            x0 += w;
        }
        y0 += h;
    }
    Ok(())
}

fn render_terminal_bootstrap(width: usize, height: usize) -> Vec<u32> {
    let mut px = vec![0u32; width * height];
    for y in 0..height {
        let row = y * width;
        for x in 0..width {
            let shade = (((x + y) % 24) as u32) * 2;
            let c = 0xFF00_0000 | ((18 + shade) << 16) | ((20 + shade) << 8) | (24 + shade);
            px[row + x] = c;
        }
    }

    fill_rect(&mut px, width, 0, 0, width as i32, 34, 0xFF1D_2330);
    fill_rect(
        &mut px,
        width,
        10,
        8,
        width as i32 - 20,
        height as i32 - 18,
        0xFF0D_1117,
    );
    draw_text(&mut px, width, 16, 10, "Terminal (bootstrap)", 0xFFCF_D8E3);
    draw_text(
        &mut px,
        width,
        24,
        48,
        "Window init OK. Next: terminal emulator core.",
        0xFFA6_B3C2,
    );
    draw_text(
        &mut px,
        width,
        24,
        68,
        "Press Esc to close this test window.",
        0xFF7D_8CA1,
    );
    px
}

fn fill_rect(px: &mut [u32], stride: usize, x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 {
        return;
    }
    let height = px.len() / stride;
    let x0 = x.max(0) as usize;
    let y0 = y.max(0) as usize;
    let x1 = (x + w).max(0) as usize;
    let y1 = (y + h).max(0) as usize;
    let x1 = x1.min(stride);
    let y1 = y1.min(height);
    for yy in y0..y1 {
        let row = yy * stride;
        for xx in x0..x1 {
            px[row + xx] = color;
        }
    }
}

fn draw_text(px: &mut [u32], stride: usize, x: i32, y: i32, s: &str, color: u32) {
    let mut pen_x = x;
    for ch in s.bytes() {
        draw_char(px, stride, pen_x, y, ch, color);
        pen_x += 8;
    }
}

fn draw_char(px: &mut [u32], stride: usize, x: i32, y: i32, ch: u8, color: u32) {
    let glyph = tiny_glyph(ch);
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..6 {
            if (bits >> (7 - col)) & 1 == 0 {
                continue;
            }
            let xx = x + col;
            let yy = y + row as i32;
            if xx < 0 || yy < 0 {
                continue;
            }
            let xx = xx as usize;
            let yy = yy as usize;
            let height = px.len() / stride;
            if xx >= stride || yy >= height {
                continue;
            }
            px[yy * stride + xx] = color;
        }
    }
}

fn tiny_glyph(ch: u8) -> [u8; 8] {
    match ch {
        b'A'..=b'Z' => [0x30, 0x48, 0x84, 0xFC, 0x84, 0x84, 0x84, 0x00],
        b'a'..=b'z' => [0x00, 0x00, 0x78, 0x04, 0x7C, 0x84, 0x7C, 0x00],
        b'0'..=b'9' => [0x78, 0x84, 0x8C, 0x94, 0xA4, 0x84, 0x78, 0x00],
        b'.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00],
        b':' => [0x00, 0x30, 0x30, 0x00, 0x00, 0x30, 0x30, 0x00],
        b'-' => [0x00, 0x00, 0x00, 0x7C, 0x00, 0x00, 0x00, 0x00],
        b' ' => [0x00; 8],
        _ => [0x7C, 0x84, 0x18, 0x30, 0x30, 0x00, 0x30, 0x00],
    }
}

fn find_kagami_tid() -> Option<u64> {
    for name in KAGAMI_PROCESS_CANDIDATES {
        if let Some(tid) = find_process_by_name(name) {
            return Some(tid);
        }
    }
    None
}

fn parse_kagami_tid_from_args() -> Option<u64> {
    for arg in std::env::args().skip(1) {
        if let Some(rest) = arg.strip_prefix("--kagami-tid=")
            && let Ok(tid) = rest.parse::<u64>()
            && tid != 0
        {
            return Some(tid);
        }
    }
    None
}
