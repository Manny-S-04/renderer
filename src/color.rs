pub type Color = u32;

pub const BLACK: Color = 0xFF000000;
pub const WHITE: Color = 0xFFFFFFFF;
pub const RED: Color = 0xFFFF0000;
pub const GREEN: Color = 0xFF00FF00;
pub const BLUE: Color = 0xFF0000FF;

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
}

pub const fn channels(c: Color) -> [u8; 4] {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8, (c >> 24) as u8]
}

fn mix_channel(s: u8, d: u8, a: u8) -> u8 {
    let a = a as u32;
    ((s as u32 * a + d as u32 * (255 - a) + 127) / 255) as u8
}

pub fn blend(dst: Color, src: Color) -> Color {
    let [sr, sg, sb, sa] = channels(src);
    if sa == 0xFF {
        return src;
    }
    if sa == 0 {
        return dst;
    }
    let [dr, dg, db, da] = channels(dst);
    rgba(
        mix_channel(sr, dr, sa),
        mix_channel(sg, dg, sa),
        mix_channel(sb, db, sa),
        da,
    )
}

fn to_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

pub fn mix3(colors: [Color; 3], w: [f32; 3]) -> Color {
    let c0 = channels(colors[0]);
    let c1 = channels(colors[1]);
    let c2 = channels(colors[2]);
    let mut out = [0u8; 4];
    for i in 0..4 {
        out[i] = to_u8(c0[i] as f32 * w[0] + c1[i] as f32 * w[1] + c2[i] as f32 * w[2]);
    }
    rgba(out[0], out[1], out[2], out[3])
}

pub fn shade(c: Color, k: f32) -> Color {
    let [r, g, b, a] = channels(c);
    rgba(
        to_u8(r as f32 * k),
        to_u8(g as f32 * k),
        to_u8(b as f32 * k),
        a,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_alpha() {
        assert_eq!(blend(BLACK, WHITE), WHITE);
        assert_eq!(blend(BLACK, 0x00FFFFFF), BLACK);
        assert_eq!(
            blend(BLACK, rgba(255, 255, 255, 128)),
            rgba(128, 128, 128, 255)
        );
    }

    #[test]
    fn mix3_vertex() {
        assert_eq!(mix3([RED, GREEN, BLUE], [1.0, 0.0, 0.0]), RED);
        assert_eq!(mix3([RED, GREEN, BLUE], [0.0, 0.0, 1.0]), BLUE);
    }
}
