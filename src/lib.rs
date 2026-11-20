//! Fluxxy's PvP Pack: three small client-side helpers in one mod, each with its own switch in
//! Options > Mods > Fluxxy's PvP Pack.
//!
//! - **Low fire**: the burning overlay drawn lower on the screen, so it covers less of the view.
//! - **Food info**: the saturation you have left drawn over the hunger bar, and how close the next
//!   hunger point is to going (exhaustion), as a thin bar above it.
//! - **Elytra pitch**: while gliding, the angle you look at, under the crosshair.
//!
//! Everything here is drawn by the mod itself: no texture or code of another mod is used.
use mod_api::{
    BlockScreenEffect, Host, BlockScreenEffectDrawEvent, ClientHost, ExtractBlockScreenEffectEvent, Flow,
    GuiLayerPostEvent, Hooks, Registrar, RegistrationContext, RenderGuiPostEvent, SettingSpec,
};

/// The mod's id: its settings folder.
pub const ID: &str = "fluxxy_pvp";

/// The settings on the mod's page in Options > Mods.
pub fn settings() -> [SettingSpec; 4] {
    [
        SettingSpec::toggle("low_fire", "fluxxy_pvp.setting.low_fire", true)
            .tooltip("fluxxy_pvp.setting.low_fire.tooltip"),
        SettingSpec::choice(
            "fire_height",
            "fluxxy_pvp.setting.fire_height",
            FIRE_HEIGHTS.map(|(name, _)| format!("fluxxy_pvp.fire_height.{name}")),
            1,
        )
        .tooltip("fluxxy_pvp.setting.fire_height.tooltip"),
        SettingSpec::toggle("food_info", "fluxxy_pvp.setting.food_info", true)
            .tooltip("fluxxy_pvp.setting.food_info.tooltip"),
        SettingSpec::toggle("elytra_pitch", "fluxxy_pvp.setting.elytra_pitch", true)
            .tooltip("fluxxy_pvp.setting.elytra_pitch.tooltip"),
    ]
}

// ---------------------------------------------------------------------------------------------
// Low fire
// ---------------------------------------------------------------------------------------------

/// How far down the fire goes, in the overlay's own units: vanilla draws it 0.3 below the middle
/// of the view, so 0.3 is vanilla's place.
pub const FIRE_HEIGHTS: [(&str, f32); 3] = [("low", 0.5), ("lower", 0.65), ("lowest", 0.8)];
/// Vanilla's fire overlay: two quads of the fire sprite, 0.24 to each side and turned 10 degrees
/// toward the middle, at 90% opacity (`ScreenEffectRenderer.renderFire`).
const FIRE_SIDE: f32 = 0.24;
const FIRE_TURN_DEGREES: f32 = 10.0;
const FIRE_INK: u32 = 0xE6FF_FFFF;
const FIRE_SPRITE: &str = "block/fire_1";

/// The four corners of one of the two fire quads (`side` -1 for the left one, 1 for the right
/// one), moved `drop` down: top left, bottom left, bottom right, top right.
pub fn fire_quad(side: f32, drop: f32) -> [[f32; 3]; 4] {
    let (sin, cos) = (side * FIRE_TURN_DEGREES).to_radians().sin_cos();
    // A corner of the unit quad at z = -0.5, turned about the vertical axis, then moved.
    let corner = |x: f32, y: f32| {
        let z = -0.5;
        [x * cos + z * sin - side * FIRE_SIDE, y - drop, -x * sin + z * cos]
    };
    [
        corner(-0.5, 0.5),
        corner(-0.5, -0.5),
        corner(0.5, -0.5),
        corner(0.5, 0.5),
    ]
}

// ---------------------------------------------------------------------------------------------
// Food info
// ---------------------------------------------------------------------------------------------

/// The food row as vanilla lays it out (`Gui.renderFood`): ten 9-pixel icons, 8 pixels apart,
/// right to left from 91 pixels right of the middle, 39 pixels above the bottom.
const FOOD_RIGHT: f32 = 91.0;
const FOOD_ABOVE_BOTTOM: f32 = 39.0;
const ICON: f32 = 9.0;
const ICON_STEP: f32 = 8.0;
/// Saturation's outline: gold, as players know it from food mods.
const SATURATION_INK: u32 = 0xFFFF_D700;
/// Exhaustion's bar: translucent white, 1 pixel high, just above the row.
const EXHAUSTION_INK: u32 = 0x80FF_FFFF;
/// Exhaustion takes a hunger point (or saturation) away when it reaches 4 (`FoodData.tick`).
const EXHAUSTION_MAX: f32 = 4.0;

/// How full each of the ten saturation outlines is, 0 to 1, the rightmost icon first: each icon
/// is 2 points of saturation, as each icon is 2 points of food.
pub fn saturation_icons(saturation: f32) -> [f32; 10] {
    std::array::from_fn(|i| ((saturation - 2.0 * i as f32) / 2.0).clamp(0.0, 1.0))
}

/// How much of the row's width the exhaustion bar takes, 0 to 1.
pub fn exhaustion_share(exhaustion: f32) -> f32 {
    (exhaustion / EXHAUSTION_MAX).clamp(0.0, 1.0)
}

/// The rectangles `[x, y, w, h]` that draw the food info on a GUI of `size`: an outline around
/// the part of each icon the saturation fills, then the exhaustion bar.
pub fn food_rects(size: [f32; 2], saturation: f32, exhaustion: f32) -> Vec<([f32; 4], u32)> {
    let right = (size[0] / 2.0).floor() + FOOD_RIGHT;
    let top = size[1] - FOOD_ABOVE_BOTTOM;
    let mut rects = Vec::new();
    for (i, fill) in saturation_icons(saturation).into_iter().enumerate() {
        if fill <= 0.0 {
            continue;
        }
        // The icon's box, filled from its right edge as vanilla fills half shanks.
        let width = (ICON * fill).max(1.0);
        let x = right - i as f32 * ICON_STEP - ICON + (ICON - width);
        rects.push(([x, top, width, 1.0], SATURATION_INK));
        rects.push(([x, top + ICON - 1.0, width, 1.0], SATURATION_INK));
        rects.push(([x + width - 1.0, top, 1.0, ICON], SATURATION_INK));
        if fill >= 1.0 {
            rects.push(([x, top, 1.0, ICON], SATURATION_INK));
        }
    }
    let share = exhaustion_share(exhaustion);
    if share > 0.0 {
        let row = ICON_STEP * 9.0 + ICON;
        let width = row * share;
        rects.push(([right - width, top - 2.0, width, 1.0], EXHAUSTION_INK));
    }
    rects
}

// ---------------------------------------------------------------------------------------------
// Elytra pitch
// ---------------------------------------------------------------------------------------------

/// White text with vanilla's shadow.
const TEXT_INK: u32 = 0xFFFF_FFFF;
/// Pixels below the middle of the screen: under the crosshair.
const PITCH_BELOW: f32 = 12.0;

/// What shows under the crosshair: the angle above (+) or below (-) the horizon. The game's
/// pitch is positive looking down, so its sign is turned.
pub fn pitch_text(pitch: f32) -> String {
    let up = -pitch;
    let up = if up.abs() < 0.05 { 0.0 } else { up };
    format!("{up:+.1}°")
}

// ---------------------------------------------------------------------------------------------
// The hooks
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
struct Pack;

fn toggle(host: &dyn ClientHost, key: &str) -> bool {
    host.on(ID) && host.setting_bool(key).unwrap_or(true)
}

#[mod_api::horizon_mod]
impl Hooks for Pack {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        for setting in settings() {
            // Only a malformed spec is refused, and the tests check every spec.
            let _ = r.register_setting(setting);
        }
    }

    /// The fire overlay is drawn by this mod instead (`block_screen_effect_draw`).
    fn extract_block_screen_effect(
        &self,
        host: &dyn ClientHost,
        event: &mut ExtractBlockScreenEffectEvent,
    ) -> Flow {
        if event.effect == BlockScreenEffect::Fire && toggle(host, "low_fire") {
            event.custom_draw = true;
        }
        Flow::Continue
    }

    /// Vanilla's two fire quads, lower.
    fn block_screen_effect_draw(
        &self,
        host: &dyn ClientHost,
        event: &mut BlockScreenEffectDrawEvent<'_>,
    ) {
        if event.effect != BlockScreenEffect::Fire || !toggle(host, "low_fire") {
            return;
        }
        let height = host
            .setting_choice("fire_height")
            .map_or(1, |i| i as usize)
            .min(FIRE_HEIGHTS.len() - 1);
        let drop = FIRE_HEIGHTS[height].1;
        let uv = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        let mut drawn = true;
        for side in [-1.0, 1.0] {
            // The right quad shows the sprite mirrored, as vanilla's does.
            let uv = if side > 0.0 { uv.map(|[u, v]| [1.0 - u, v]) } else { uv };
            drawn &= event.draw.sprite(FIRE_SPRITE, fire_quad(side, drop), uv, FIRE_INK);
        }
        // Without the sprite nothing was drawn: leave vanilla's fire rather than none.
        event.replace_vanilla = drawn;
    }

    /// Saturation and exhaustion over the hunger bar.
    fn gui_layer_post(&self, host: &dyn ClientHost, event: &mut GuiLayerPostEvent<'_>) {
        if event.layer != "minecraft:player_health" || !toggle(host, "food_info") {
            return;
        }
        let Some(player) = host.local_player() else {
            return;
        };
        let world: &dyn Host = host;
        let survival = player.is_creative(world) == Some(false)
            && player.is_spectator(world) == Some(false);
        // On a mount with health, its hearts take the food row's place.
        let mounted = player
            .entity()
            .vehicle(world)
            .and_then(|v| host.living(v))
            .is_some_and(|v| v.health(world).is_some());
        if !survival || mounted {
            return;
        }
        let Some(food) = player.food(world) else {
            return;
        };
        let draw = &mut *event.draw;
        for (rect, ink) in food_rects(draw.size(), food.saturation, food.exhaustion) {
            draw.fill(rect, ink);
        }
    }

    /// The pitch under the crosshair while gliding.
    fn render_gui_post(&self, host: &dyn ClientHost, event: &mut RenderGuiPostEvent<'_>) {
        if !toggle(host, "elytra_pitch") {
            return;
        }
        let Some(player) = host.local_player() else {
            return;
        };
        let world: &dyn Host = host;
        let gliding = host.living(player.entity()).and_then(|l| l.is_gliding(world));
        if gliding != Some(true) {
            return;
        }
        let Some([_, pitch]) = player.entity().rotation(world) else {
            return;
        };
        let text = pitch_text(pitch);
        let draw = &mut *event.draw;
        let [w, h] = draw.size();
        let x = (w - draw.text_width(&text)) / 2.0;
        draw.text(&text, [x, h / 2.0 + PITCH_BELOW], TEXT_INK, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_setting_is_one_the_game_takes() {
        let keys: Vec<String> = settings().iter().map(|s| format!("{s:?}")).collect();
        assert_eq!(keys.len(), 4);
    }

    #[test]
    fn every_setting_has_a_name_in_english() {
        let lang = include_str!("../assets/fluxxy_pvp/lang/en_us.json");
        for key in [
            "low_fire",
            "fire_height",
            "food_info",
            "elytra_pitch",
        ] {
            assert!(lang.contains(&format!("\"fluxxy_pvp.setting.{key}\"")), "{key}");
            assert!(lang.contains(&format!("\"fluxxy_pvp.setting.{key}.tooltip\"")), "{key}");
        }
        for (name, _) in FIRE_HEIGHTS {
            assert!(lang.contains(&format!("\"fluxxy_pvp.fire_height.{name}\"")), "{name}");
        }
    }

    #[test]
    fn fire_goes_down_by_the_drop_and_keeps_its_size() {
        let quad = fire_quad(-1.0, 0.65);
        assert_eq!(quad[0][1], 0.5 - 0.65);
        assert_eq!(quad[1][1], -0.5 - 0.65);
        // Turned about the vertical axis only: the quad stays 1 high and about 1 wide.
        let width = ((quad[2][0] - quad[1][0]).powi(2) + (quad[2][2] - quad[1][2]).powi(2)).sqrt();
        assert!((width - 1.0).abs() < 1e-5, "{width}");
    }

    #[test]
    fn the_two_fire_quads_mirror_each_other() {
        let (left, right) = (fire_quad(-1.0, 0.5), fire_quad(1.0, 0.5));
        for i in 0..4 {
            let mirrored = right[3 - (i ^ 1)];
            assert!((left[i][0] + mirrored[0]).abs() < 1e-5);
            assert!((left[i][2] - mirrored[2]).abs() < 1e-5);
        }
    }

    #[test]
    fn saturation_fills_two_points_an_icon_from_the_right() {
        let icons = saturation_icons(5.0);
        assert_eq!(&icons[..4], &[1.0, 1.0, 0.5, 0.0]);
        assert_eq!(saturation_icons(20.0), [1.0; 10]);
        assert_eq!(saturation_icons(0.0), [0.0; 10]);
    }

    #[test]
    fn exhaustion_is_a_share_of_four() {
        assert_eq!(exhaustion_share(0.0), 0.0);
        assert_eq!(exhaustion_share(1.0), 0.25);
        assert_eq!(exhaustion_share(7.0), 1.0);
    }

    #[test]
    fn the_food_info_sits_on_vanillas_food_row() {
        // A 320 x 240 GUI: the row's right edge is at 160 + 91, its top at 240 - 39.
        let rects = food_rects([320.0, 240.0], 2.0, 0.0);
        let right = rects.iter().map(|(r, _)| r[0] + r[2]).fold(0.0, f32::max);
        assert_eq!(right, 251.0);
        assert!(rects.iter().all(|(r, _)| r[1] >= 201.0 && r[1] + r[3] <= 210.0));
        // Nothing to show: nothing drawn.
        assert!(food_rects([320.0, 240.0], 0.0, 0.0).is_empty());
        // The exhaustion bar ends at the row's right edge, above it.
        let bar = food_rects([320.0, 240.0], 0.0, 2.0);
        assert_eq!(bar, vec![([251.0 - 40.5, 199.0, 40.5, 1.0], EXHAUSTION_INK)]);
    }

    #[test]
    fn pitch_reads_up_as_positive() {
        assert_eq!(pitch_text(-30.0), "+30.0°");
        assert_eq!(pitch_text(12.34), "-12.3°");
        assert_eq!(pitch_text(0.01), "+0.0°");
    }
}
