//! Fluxxy's PvP Pack: client-side PvP helpers in one mod, each with its own switch in
//! Options > Mods > Fluxxy's PvP Pack.
//!
//! - **Low fire**: the burning overlay drawn lower on the screen, so it covers less of the view.
//! - **Food info**: AppleSkin's saturation outlines over the hunger bar and its exhaustion bar
//!   under it, with AppleSkin's own pixels (public domain).
//! - **Elytra pitch**: while gliding, the angle you look at, under the crosshair.
//! - **Armor HUD**: your four armour pieces, with their durability bars, in slots left of the
//!   hotbar. They are only drawn: nothing can be clicked or moved there.
//! - **Item counters**: how many arrows, golden apples, ender pearls, cobwebs, totems and potions
//!   you carry, in a column of slots in the bottom right corner.
//! - **Damage indicator**: how much health a hit took, above the hearts, for a moment.
//!
//! Everything here is drawn by the mod itself: no texture or code of another mod is used.
use std::sync::Mutex;

mod appleskin_icons;

use mod_api::{
    EquipmentSlot, ItemStack, RenderFramePreEvent,
    BlockScreenEffect, Host, BlockScreenEffectDrawEvent, ClientHost, ExtractBlockScreenEffectEvent, Flow,
    GuiLayerPostEvent, GuiLayerPreEvent, Hooks, Registrar, RegistrationContext, RenderGuiPostEvent, SettingSpec,
};

/// The mod's id: its settings folder.
pub const ID: &str = "fluxxy_pvp";

/// The settings on the mod's page in Options > Mods.
pub fn settings() -> [SettingSpec; 7] {
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
        SettingSpec::toggle("armor_hud", "fluxxy_pvp.setting.armor_hud", true)
            .tooltip("fluxxy_pvp.setting.armor_hud.tooltip"),
        SettingSpec::toggle("item_counters", "fluxxy_pvp.setting.item_counters", true)
            .tooltip("fluxxy_pvp.setting.item_counters.tooltip"),
        SettingSpec::toggle("damage_indicator", "fluxxy_pvp.setting.damage_indicator", true)
            .tooltip("fluxxy_pvp.setting.damage_indicator.tooltip"),
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
/// Exhaustion takes a hunger point (or saturation) away when it reaches 4 (`FoodData.tick`).
const EXHAUSTION_MAX: f32 = 4.0;
/// The exhaustion bar's opacity, as AppleSkin draws it (0.75).
const EXHAUSTION_ALPHA: u32 = 0xBF;

/// The saturation outlines AppleSkin draws over the food row: `(icon x, variant)` with `variant`
/// 0 to 3 the quarter, half, three-quarter and whole outline of `appleskin_icons::SATURATION`.
/// Each icon is 2 points of saturation, from the right (`HUDOverlayHandler.drawSaturationOverlay`).
pub fn saturation_icons(right: f32, saturation: f32) -> Vec<(f32, usize)> {
    let saturation = saturation.clamp(0.0, 20.0);
    let icons = (saturation / 2.0).ceil() as usize;
    (0..icons)
        .map(|i| {
            let part = saturation / 2.0 - i as f32;
            let variant = if part >= 1.0 {
                3
            } else if part > 0.5 {
                2
            } else if part > 0.25 {
                1
            } else {
                0
            };
            (right - i as f32 * ICON_STEP - ICON, variant)
        })
        .collect()
}

/// How many of the exhaustion bar's 81 pixels show, from its right edge
/// (`HUDOverlayHandler.drawExhaustionOverlay`).
pub fn exhaustion_width(exhaustion: f32) -> u8 {
    ((exhaustion / EXHAUSTION_MAX).clamp(0.0, 1.0) * 81.0) as u8
}

/// The food row's right edge and top on a GUI of `size`.
fn food_row(size: [f32; 2]) -> (f32, f32) {
    ((size[0] / 2.0).floor() + FOOD_RIGHT, size[1] - FOOD_ABOVE_BOTTOM)
}

/// Draw runs of pixels at `at`, keeping only the columns from `from` on, times `alpha` (0-255).
fn draw_runs(
    draw: &mut dyn mod_api::DrawContext,
    runs: &[(u8, u8, u8, u32)],
    at: [f32; 2],
    from: u8,
    alpha: u32,
) {
    for &(x, y, w, argb) in runs {
        let (start, end) = (x.max(from), x + w);
        if start >= end {
            continue;
        }
        let a = ((argb >> 24) * alpha / 255) << 24;
        let rect = [at[0] + start as f32, at[1] + y as f32, (end - start) as f32, 1.0];
        draw.fill(rect, a | (argb & 0x00FF_FFFF));
    }
}

/// Whether the food row shows: the local player in survival or adventure, not on a mount whose
/// hearts take its place. Their food when it does.
fn shown_food(host: &dyn ClientHost) -> Option<mod_api::PlayerFood> {
    let player = host.local_player()?;
    let world: &dyn Host = host;
    let survival =
        player.is_creative(world) == Some(false) && player.is_spectator(world) == Some(false);
    let mounted = player
        .entity()
        .vehicle(world)
        .and_then(|v| host.living(v))
        .is_some_and(|v| v.health(world).is_some());
    if !survival || mounted {
        return None;
    }
    player.food(world)
}

// ---------------------------------------------------------------------------------------------
// Elytra pitch
// ---------------------------------------------------------------------------------------------

/// White text with vanilla's shadow.
const TEXT_INK: u32 = 0xFFFF_FFFF;
/// GUI pixels below the middle of the screen: under the crosshair, clear of it.
const PITCH_BELOW: f32 = 44.0;

/// What shows under the crosshair: the angle above the horizon in whole degrees, positive
/// looking up and negative looking down. The game's pitch is the other way round (positive
/// looking down), so its sign is turned.
pub fn pitch_text(pitch: f32) -> String {
    let degrees = (-pitch).round() as i32;
    format!("{degrees}°")
}

// ---------------------------------------------------------------------------------------------
// Armor HUD and item counters
// ---------------------------------------------------------------------------------------------

/// Vanilla's hotbar: 182 pixels wide, 22 high, its slots 20 apart from 1 in; an item sits 3 in
/// from a slot's corner (`Gui.renderItemHotbar`). The off hand's slot takes 29 pixels left of it.
const HOTBAR_HALF: f32 = 91.0;
const HOTBAR_HEIGHT: f32 = 22.0;
const SLOT: f32 = 20.0;
const OFFHAND: f32 = 29.0;
const GAP: f32 = 4.0;
const HOTBAR_SPRITE: [&str; 2] = ["hud/hotbar", "gui/sprites/hud/hotbar"];
/// Drawn when the hotbar sprite can't be found: vanilla's slot colours.
const SLOT_EDGE: u32 = 0xFF8B_8B8B;
const SLOT_FILL: u32 = 0xC037_3737;

/// The armour slots, left to right.
pub const ARMOR: [EquipmentSlot; 4] = [
    EquipmentSlot::Feet,
    EquipmentSlot::Legs,
    EquipmentSlot::Chest,
    EquipmentSlot::Head,
];

/// The armour row: `[x, y]` of its top left on a GUI of `size`, left of the off hand's slot.
pub fn armor_row(size: [f32; 2]) -> [f32; 2] {
    let width = ARMOR.len() as f32 * SLOT + 2.0;
    let x = (size[0] / 2.0).floor() - HOTBAR_HALF - OFFHAND - GAP - width;
    [x, size[1] - HOTBAR_HEIGHT]
}

/// What the counters count, bottom to top, each a group of item ids.
pub const COUNTED: [(&str, &[&str]); 6] = [
    ("golden_apples", &["minecraft:golden_apple", "minecraft:enchanted_golden_apple"]),
    ("ender_pearls", &["minecraft:ender_pearl"]),
    ("totems", &["minecraft:totem_of_undying"]),
    ("arrows", &["minecraft:arrow", "minecraft:spectral_arrow", "minecraft:tipped_arrow"]),
    ("potions", &["minecraft:potion", "minecraft:splash_potion", "minecraft:lingering_potion"]),
    ("cobwebs", &["minecraft:cobweb"]),
];

/// Totals of each group of `COUNTED` among `(item id, count)` pairs, in its order.
pub fn count_groups<'a>(items: impl IntoIterator<Item = (&'a str, u32)>) -> [u32; 6] {
    let mut totals = [0; 6];
    for (id, count) in items {
        if let Some(group) = COUNTED.iter().position(|(_, ids)| ids.contains(&id)) {
            totals[group] += count;
        }
    }
    totals
}

/// The top left of the `n`th counter slot from the bottom, in the bottom right corner.
pub fn counter_slot(size: [f32; 2], n: usize) -> [f32; 2] {
    let x = size[0] - SLOT - 4.0;
    [x, size[1] - HOTBAR_HEIGHT - n as f32 * (SLOT + 2.0)]
}

/// One slot's frame, `SLOT + 2` square, with its top left at `at`: the hotbar's own slot when
/// the sprite is there, else vanilla's slot colours.
fn slot_frame(draw: &mut dyn mod_api::DrawContext, at: [f32; 2], slots: usize) {
    let width = slots as f32 * SLOT + 2.0;
    let left = [0.0, 0.0, width - 1.0, HOTBAR_HEIGHT];
    let right = [181.0, 0.0, 1.0, HOTBAR_HEIGHT];
    for name in HOTBAR_SPRITE {
        if draw.sprite(name, left, [at[0], at[1], width - 1.0, HOTBAR_HEIGHT], None) {
            draw.sprite(name, right, [at[0] + width - 1.0, at[1], 1.0, HOTBAR_HEIGHT], None);
            return;
        }
    }
    draw.fill([at[0], at[1], width, HOTBAR_HEIGHT], SLOT_EDGE);
    for i in 0..slots {
        let x = at[0] + 2.0 + i as f32 * SLOT;
        draw.fill([x, at[1] + 2.0, SLOT - 2.0, HOTBAR_HEIGHT - 4.0], SLOT_FILL);
    }
}

// ---------------------------------------------------------------------------------------------
// Damage indicator
// ---------------------------------------------------------------------------------------------

/// Seconds a hit's number stays.
const DAMAGE_SECONDS: f64 = 1.5;
/// Red, as the hearts.
const DAMAGE_INK: u32 = 0xFFFF_5555;

/// What shows for a hit that took `health` points: half hearts, as the hearts count.
pub fn damage_text(health: f32) -> String {
    let hearts = (health / 2.0 * 10.0).round() / 10.0;
    format!("-{hearts}❤")
}

/// The local player's health between frames, and the last hit's number while it shows.
#[derive(Default)]
pub struct DamageWatch {
    last: Option<f32>,
    shown: Option<(f32, f64)>,
}

impl DamageWatch {
    /// A frame `seconds` long with the player at `health` (None outside a world).
    pub fn frame(&mut self, health: Option<f32>, seconds: f64) {
        if let Some((_, left)) = &mut self.shown {
            *left -= seconds;
            if *left <= 0.0 {
                self.shown = None;
            }
        }
        if let (Some(before), Some(now)) = (self.last, health)
            && now < before - 0.01
        {
            // Hits close together add up, as one burst.
            let total = self.shown.map_or(0.0, |(taken, _)| taken) + (before - now);
            self.shown = Some((total, DAMAGE_SECONDS));
        }
        self.last = health;
    }

    /// The health the hits on show took.
    pub fn shown(&self) -> Option<f32> {
        self.shown.map(|(taken, _)| taken)
    }
}

// ---------------------------------------------------------------------------------------------
// The hooks
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
struct Pack {
    // Never held across a host call: a host call can fire another of this mod's hooks.
    damage: Mutex<DamageWatch>,
}

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

    /// AppleSkin's exhaustion bar, under the food row.
    fn gui_layer_pre(&self, host: &dyn ClientHost, event: &mut GuiLayerPreEvent<'_>) -> Flow {
        if event.layer == "minecraft:player_health"
            && toggle(host, "food_info")
            && let Some(food) = shown_food(host)
        {
            let draw = &mut *event.draw;
            let (right, top) = food_row(draw.size());
            let from = 81 - exhaustion_width(food.exhaustion);
            let at = [right - 81.0, top];
            draw_runs(draw, appleskin_icons::EXHAUSTION, at, from, EXHAUSTION_ALPHA);
        }
        Flow::Continue
    }

    /// AppleSkin's saturation outlines, over the food row.
    fn gui_layer_post(&self, host: &dyn ClientHost, event: &mut GuiLayerPostEvent<'_>) {
        if event.layer != "minecraft:player_health" || !toggle(host, "food_info") {
            return;
        }
        let Some(food) = shown_food(host) else {
            return;
        };
        let draw = &mut *event.draw;
        let (right, top) = food_row(draw.size());
        for (x, variant) in saturation_icons(right, food.saturation) {
            draw_runs(draw, appleskin_icons::SATURATION[variant], [x, top], 0, 255);
        }
    }

    /// Every frame: the player's health, for the damage indicator.
    fn render_frame_pre(&self, host: &mut dyn ClientHost, event: &mut RenderFramePreEvent) {
        let health = {
            let host: &dyn ClientHost = host;
            let world: &dyn Host = host;
            host.local_player()
                .and_then(|p| host.living(p.entity()))
                .and_then(|l| l.health(world))
        };
        let seconds = event.elapsed.as_secs_f64();
        self.damage.lock().unwrap_or_else(|e| e.into_inner()).frame(health, seconds);
    }

    /// The armour row, the counters, the damage taken and the pitch, over the HUD.
    fn render_gui_post(&self, host: &dyn ClientHost, event: &mut RenderGuiPostEvent<'_>) {
        if !host.on(ID) {
            return;
        }
        let Some(player) = host.local_player() else {
            return;
        };
        let world: &dyn Host = host;
        if player.is_spectator(world) == Some(true) {
            return;
        }
        let draw = &mut *event.draw;
        let size = draw.size();
        let living = host.living(player.entity());
        if toggle(host, "armor_hud")
            && let Some(living) = living
        {
            // A slot only for a piece worn, in its own place: nothing worn, no row.
            let at = armor_row(size);
            for (i, slot) in ARMOR.into_iter().enumerate() {
                if let Some(stack) = living.equipment(world, slot) {
                    let x = at[0] + i as f32 * SLOT;
                    slot_frame(draw, [x, at[1]], 1);
                    draw.item_stack(&stack, [x + 3.0, at[1] + 3.0], 16.0);
                }
            }
        }
        if toggle(host, "item_counters") {
            let mut stacks: Vec<ItemStack> = player.inventory(world).unwrap_or_default();
            if let Some(off) = living.and_then(|l| l.equipment(world, EquipmentSlot::OffHand)) {
                stacks.push(off);
            }
            let ids: Vec<(String, u32)> = stacks
                .iter()
                .map(|s| (s.item().id(world), s.count()))
                .collect();
            let totals = count_groups(ids.iter().map(|(id, n)| (id.as_str(), *n)));
            let mut n = 0;
            for (group, (_, members)) in COUNTED.iter().enumerate() {
                if totals[group] == 0 {
                    continue;
                }
                let Some(i) = ids.iter().position(|(id, _)| members.contains(&id.as_str())) else {
                    continue;
                };
                let at = counter_slot(size, n);
                slot_frame(draw, at, 1);
                let text = totals[group].to_string();
                draw.gui_draw_item_with(&stacks[i], [at[0] + 3.0, at[1] + 3.0], Some(&text));
                n += 1;
            }
        }
        if toggle(host, "damage_indicator")
            && let Some(taken) = self.damage.lock().unwrap_or_else(|e| e.into_inner()).shown()
        {
            // Above the hearts: vanilla's health row is 39 pixels up, left of the middle.
            let text = damage_text(taken);
            let x = (size[0] / 2.0).floor() - HOTBAR_HALF;
            draw.text(&text, [x, size[1] - 39.0 - 22.0], DAMAGE_INK, true);
        }
        if !toggle(host, "elytra_pitch") {
            return;
        }
        let gliding = host.living(player.entity()).and_then(|l| l.is_gliding(world));
        if gliding != Some(true) {
            return;
        }
        let Some([_, pitch]) = player.entity().rotation(world) else {
            return;
        };
        let text = pitch_text(pitch);
        let [w, h] = size;
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
        assert_eq!(keys.len(), 7);
    }

    #[test]
    fn every_setting_has_a_name_in_english() {
        let lang = include_str!("../assets/fluxxy_pvp/lang/en_us.json");
        for key in [
            "low_fire",
            "fire_height",
            "food_info",
            "elytra_pitch",
            "armor_hud",
            "item_counters",
            "damage_indicator",
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
    fn saturation_takes_appleskins_outlines_from_the_right() {
        // 5 saturation: two whole icons, then half of one (0.5 is not over a half: the quarter's
        // three-quarter outline is for over 0.5).
        let icons = saturation_icons(251.0, 5.0);
        assert_eq!(icons, vec![(242.0, 3), (234.0, 3), (226.0, 1)]);
        assert_eq!(saturation_icons(251.0, 0.0), vec![]);
        assert_eq!(saturation_icons(251.0, 20.0).len(), 10);
        assert_eq!(saturation_icons(251.0, 1.6), vec![(242.0, 2)]);
        assert_eq!(saturation_icons(251.0, 0.3), vec![(242.0, 0)]);
    }

    #[test]
    fn exhaustion_fills_the_row_as_it_nears_four() {
        assert_eq!(exhaustion_width(0.0), 0);
        assert_eq!(exhaustion_width(2.0), 40);
        assert_eq!(exhaustion_width(9.0), 81);
    }

    #[test]
    fn pitch_is_positive_looking_up_in_whole_degrees() {
        // The game's pitch is positive looking down.
        assert_eq!(pitch_text(-17.4), "17°");
        assert_eq!(pitch_text(30.6), "-31°");
        assert_eq!(pitch_text(-0.2), "0°");
    }

    #[test]
    fn armor_sits_left_of_the_off_hand_slot() {
        // A 500 x 281 GUI: the hotbar starts at 250 - 91, the off hand's slot 29 further left.
        let [x, y] = armor_row([500.0, 281.0]);
        assert_eq!(x + 4.0 * 20.0 + 2.0, 250.0 - 91.0 - 29.0 - 4.0);
        assert_eq!(y, 281.0 - 22.0);
    }

    #[test]
    fn counters_add_up_each_group() {
        let totals = count_groups([
            ("minecraft:golden_apple", 61),
            ("minecraft:enchanted_golden_apple", 2),
            ("minecraft:ender_pearl", 16),
            ("minecraft:ender_pearl", 16),
            ("minecraft:dirt", 64),
            ("minecraft:spectral_arrow", 3),
            ("minecraft:arrow", 64),
        ]);
        assert_eq!(totals, [63, 32, 0, 67, 0, 0]);
    }

    #[test]
    fn counters_stack_up_from_the_bottom_right() {
        let [x0, y0] = counter_slot([500.0, 281.0], 0);
        let [x1, y1] = counter_slot([500.0, 281.0], 1);
        assert_eq!((x0, x1), (476.0, 476.0));
        assert_eq!(y0, 259.0);
        assert_eq!(y0 - y1, 22.0);
    }

    #[test]
    fn a_hit_shows_for_a_moment_and_close_hits_add_up() {
        let mut watch = DamageWatch::default();
        watch.frame(Some(20.0), 0.05);
        assert_eq!(watch.shown(), None);
        watch.frame(Some(14.0), 0.05);
        assert_eq!(watch.shown(), Some(6.0));
        watch.frame(Some(11.0), 0.5);
        assert_eq!(watch.shown(), Some(9.0));
        watch.frame(Some(11.0), 1.6);
        assert_eq!(watch.shown(), None);
        // Healing shows nothing.
        watch.frame(Some(15.0), 0.05);
        assert_eq!(watch.shown(), None);
        assert_eq!(damage_text(9.0), "-4.5❤");
    }
}
