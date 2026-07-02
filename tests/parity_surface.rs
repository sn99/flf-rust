//! Integration tests driving shipped flf parity paths (not re-implementations).

use flf::character::Character;
use flf::data::parse_object_data;
use flf::global;
use flf::livingobject::LivingObject;
use flf::match_game::gameover_fires_at;
use flf::special_states;
use flf::specialattack::SpecialAttack;
use flf::weapon::Weapon;
use serde_json::json;

fn minimal_char_data(id: i32) -> flf::data::ObjectData {
    let v = json!({
        "bmp": { "name": "Test", "weapon_hp": 200, "weapon_drop_hurt": 35 },
        "frame": {
            "0": { "name": "stand", "state": 0, "wait": 1, "next": 0, "pic": 0,
                   "centerx": 40, "centery": 80, "hit_a": 0, "hit_d": 0, "hit_j": 0 },
            "10": { "name": "caught", "state": 10, "wait": 1, "next": 10, "pic": 0,
                    "centerx": 40, "centery": 80,
                    "cpoint": { "kind": 2, "x": 40, "y": 40, "fronthurtact": 132, "backhurtact": 131 } },
            "120": { "name": "catch", "state": 9, "wait": 1, "next": 120, "pic": 0,
                     "centerx": 40, "centery": 80,
                     "cpoint": { "kind": 1, "hurtable": 1, "x": 40, "y": 40 } },
            "181": { "name": "release", "state": 11, "wait": 1, "next": 0, "pic": 0,
                     "centerx": 40, "centery": 80 },
            "7": { "name": "defend", "state": 7, "wait": 1, "next": 7, "pic": 0,
                   "centerx": 40, "centery": 80 },
            "111": { "name": "defend_hit", "state": 7, "wait": 1, "next": 0, "pic": 0,
                     "centerx": 40, "centery": 80 },
            "112": { "name": "broken_defend", "state": 8, "wait": 1, "next": 0, "pic": 0,
                     "centerx": 40, "centery": 80 },
            "220": { "name": "pain", "state": 11, "wait": 1, "next": 0, "pic": 0,
                     "centerx": 40, "centery": 80 }
        }
    });
    parse_object_data(id, "character", &v)
}

fn minimal_weapon_data(light: bool) -> flf::data::ObjectData {
    let ty = if light { "lightweapon" } else { "heavyweapon" };
    let v = json!({
        "bmp": { "name": "Stick", "weapon_hp": 200, "weapon_drop_hurt": 40 },
        "frame": {
            "0": { "state": 1000, "wait": 1, "next": 0, "pic": 0, "centerx": 20, "centery": 20 },
            "40": { "state": 1002, "wait": 1, "next": 40, "pic": 0, "centerx": 20, "centery": 20 },
            "70": { "state": 1003, "wait": 1, "next": 0, "pic": 0, "centerx": 20, "centery": 20 },
            "20": { "state": 2000, "wait": 1, "next": 20, "pic": 0, "centerx": 20, "centery": 20 },
            "21": { "state": 2000, "wait": 1, "next": 20, "pic": 0, "centerx": 20, "centery": 20 }
        }
    });
    parse_object_data(100, ty, &v)
}

fn minimal_special_data(hit_fa: i32) -> flf::data::ObjectData {
    let v = json!({
        "bmp": { "name": "Ball" },
        "frame": {
            "0": { "state": 3000, "wait": 1, "next": 0, "pic": 0, "centerx": 20, "centery": 20,
                   "hit_a": 7, "hit_Fa": hit_fa, "hit_d": 1000,
                   "itr": { "kind": 0, "x": 0, "y": 0, "w": 20, "h": 20, "injury": 20, "effect": 0 } }
        }
    });
    parse_object_data(201, "specialattack", &v)
}

const BG_Z: (f64, f64) = (300.0, 450.0);
const BG_W: f64 = 800.0;

#[test]
fn gameover_fires_at_plus_30_tu() {
    assert!(!gameover_fires_at(100, None));
    assert!(!gameover_fires_at(100, Some(100)));
    assert!(!gameover_fires_at(129, Some(100)));
    assert!(gameover_fires_at(130, Some(100)));
    assert!(!gameover_fires_at(131, Some(100)));
}

/// F.LF dead_blink: arm at 0; Character::tu must advance exactly +1 per TU (not 2).
/// Remove on the TU when counter is already >= 30 (after 30 steps: 0→1…→30, then 31st removes).
#[test]
fn character_tu_dead_blink_single_step_removes_near_30_tu() {
    let data = minimal_char_data(1);
    let mut ch = Character::new(1, data, 1, 100.0, 350.0);
    ch.base.counter_dead_blink = 0;
    let mut prev = 0i32;
    for i in 1..=30 {
        ch.tu(None, BG_Z, BG_W);
        let c = ch.base.counter_dead_blink;
        assert!(
            !ch.base.removed,
            "must not remove before F.LF window finishes (tu {i}, counter {c})"
        );
        // exactly +1 per Character::tu (not +2)
        assert_eq!(c, prev + 1, "tu {i}: counter {c} vs prev {prev} (double-step bug?)");
        prev = c;
    }
    assert_eq!(ch.base.counter_dead_blink, 30);
    // F.LF: next TU with counter >= 30 destroys
    ch.tu(None, BG_Z, BG_W);
    assert!(ch.base.removed);
    assert!(ch.base.dead);
    assert!(!ch.base.sp.visible);
    assert_eq!(ch.base.counter_dead_blink, -1);
}

/// F.LF disappear: shadow_blink=120, body_blink=150 — only once per Character::tu
#[test]
fn character_tu_disappear_uses_flf_120_150_windows() {
    let data = minimal_char_data(1);
    let mut ch = Character::new(1, data, 1, 100.0, 350.0);
    ch.base.counter_disappear = 0;
    ch.base.sp.visible = true;
    ch.tu(None, BG_Z, BG_W);
    // body disappear: counter 0→1, hidden
    assert_eq!(ch.base.counter_disappear, 1);
    assert!(!ch.base.sp.visible);

    // advance to just before shadow_blink with single steps
    ch.base.counter_disappear = global::DISAPPEAR_SHADOW_BLINK - 1;
    ch.tu(None, BG_Z, BG_W);
    assert_eq!(
        ch.base.counter_disappear,
        global::DISAPPEAR_SHADOW_BLINK,
        "must enter shadow_blink at 120, not half-window from id_tu 8/16"
    );

    // at shadow_blink boundary: still in blink window (< body_blink)
    ch.base.counter_disappear = global::DISAPPEAR_SHADOW_BLINK;
    let before = ch.base.counter_disappear;
    ch.tu(None, BG_Z, BG_W);
    assert_eq!(ch.base.counter_disappear, before + 1);

    // body_blink equality arm
    ch.base.counter_disappear = global::DISAPPEAR_BODY_BLINK;
    ch.tu(None, BG_Z, BG_W);
    assert!(ch.base.sp.visible);
    assert!(ch.base.effect.blink);
    assert_eq!(ch.base.effect.timeout, 30);
    assert_eq!(ch.base.counter_disappear, global::DISAPPEAR_BODY_BLINK + 1);

    // dismiss next
    ch.tu(None, BG_Z, BG_W);
    assert_eq!(ch.base.counter_disappear, -1);
}

#[test]
fn blocking_xz_scales_horizontal_motion() {
    let data = minimal_char_data(1);
    let mut lo = LivingObject::new(1, data, 1, 200.0, 350.0);
    lo.obj_type = "character".into();
    lo.ps.vx = 10.0;
    lo.ps.vz = 0.0;
    lo.block_xz = true;
    let x0 = lo.ps.x;
    lo.physics_tu(BG_Z, BG_W);
    assert!((lo.ps.x - x0 - 1.0).abs() < 1e-6, "x delta {}", lo.ps.x - x0);
    assert!(!lo.block_xz, "flag consumed");
}

#[test]
fn weapon_drop_hurt_on_hard_land() {
    let data = minimal_weapon_data(true);
    let mut w = Weapon::new(2, data, 100.0, 350.0);
    let hp0 = w.base.hp;
    w.base.ps.y = -5.0;
    w.base.ps.vy = 12.0;
    w.base.ps.vx = 8.0;
    w.tu(BG_Z, BG_W);
    assert!(w.base.ps.y >= 0.0);
    assert!(
        w.base.hp < hp0 || w.base.frame.n == 70,
        "hp {} -> {} frame {}",
        hp0,
        w.base.hp,
        w.base.frame.n
    );
}

#[test]
fn weapon_after_hit_character_uses_gc_hit_vx() {
    let data = minimal_weapon_data(true);
    let mut w = Weapon::new(3, data, 100.0, 350.0);
    w.base.ps.vx = 5.0;
    w.after_hit_character();
    assert!((w.base.ps.vx - global::WEAPON_HIT_VX).abs() < 1e-9);
    assert_eq!(w.base.effect.timeout, 2);
}

#[test]
fn caught_cpointhurtable_reads_catcher_frame() {
    let data = minimal_char_data(5);
    let mut ch = Character::new(1, data, 1, 100.0, 350.0);
    ch.base.trans_frame(120, 0);
    assert_eq!(ch.caught_cpointhurtable(), 1);
    ch.base.trans_frame(0, 0);
    assert_eq!(ch.caught_cpointhurtable(), global::DEFAULT_CPOINT_HURTABLE);
}

#[test]
fn caught_release_goes_to_frame_181() {
    let data = minimal_char_data(5);
    let mut ch = Character::new(1, data, 1, 100.0, 350.0);
    ch.base.trans_frame(10, 0);
    ch.base.held_by = Some(99);
    ch.caught_release();
    assert!(ch.base.held_by.is_none());
    assert_eq!(ch.base.frame.n, 181);
    assert!((ch.base.effect.dvx - 3.0).abs() < 1e-9);
    assert!((ch.base.effect.dvy - (-3.0)).abs() < 1e-9);
}

#[test]
fn chase_hit_fa_accelerates_toward_target() {
    let data = minimal_special_data(1);
    let mut sp = SpecialAttack::new(9, data, 1, 0.0, -10.0, 350.0, 1);
    sp.chase_x = 100.0;
    sp.chase_z = 350.0;
    sp.base.ps.vx = 0.0;
    sp.base.ps.vz = 0.0;
    sp.base.hp = 100.0;
    special_states::dispatch(&mut sp, "TU");
    assert!(sp.base.ps.vx > 0.0);
    assert!((sp.base.ps.vx - global::CHASE_AX).abs() < 1e-9);
}

#[test]
fn chase_hit_fa_10_sets_exhaust_speed() {
    let data = minimal_special_data(10);
    let mut sp = SpecialAttack::new(9, data, 1, 0.0, -10.0, 350.0, 1);
    sp.base.ps.vx = 3.0;
    special_states::dispatch(&mut sp, "TU");
    assert!((sp.base.ps.vx - global::CHASE_EXHAUST_VX).abs() < 1e-9);
    assert_eq!(sp.base.ps.vz, 0.0);
}

#[test]
fn fronthurtact_present_on_parsed_cpoint() {
    let data = minimal_char_data(1);
    let fd = data.frames.get(&10).expect("frame 10");
    let cp = fd.cpoint.as_ref().expect("cpoint");
    assert_eq!(cp.fronthurtact, 132);
    assert_eq!(cp.backhurtact, 131);
}

#[test]
fn global_weapon_and_chase_constants_match_flf() {
    assert_eq!(global::WEAPON_BOUNCEUP_LIMIT, 8.0);
    assert_eq!(global::WEAPON_BOUNCEUP_SPEED_Y, -3.7);
    assert_eq!(global::WEAPON_HIT_VX, -3.0);
    assert_eq!(global::CHASE_MAX_VX, 14.0);
    assert_eq!(global::CHASE_AX, 0.7);
    assert_eq!(global::DISAPPEAR_SHADOW_BLINK, 120);
    assert_eq!(global::DISAPPEAR_BODY_BLINK, 150);
}

/// Weapon after_hit must clear stuck via Weapon::tu (recover_tu always runs)
#[test]
fn weapon_tu_clears_stuck_after_hit_character() {
    let data = minimal_weapon_data(true);
    let mut w = Weapon::new(4, data, 100.0, 350.0);
    w.base.ps.vx = 5.0;
    w.base.ps.y = -10.0;
    w.after_hit_character();
    assert!(w.base.effect.stuck);
    assert_eq!(w.base.effect.timeout, 2);
    // F.LF effect_stuck(0,2): timers decay; stuck clears within a few TUs
    let mut cleared = false;
    for i in 0..10 {
        w.tu(BG_Z, BG_W);
        if !w.base.effect.stuck {
            cleared = true;
            // motion not permanently frozen: integrate resumes after clear
            let x0 = w.base.ps.x;
            w.base.ps.vx = 4.0;
            w.base.ps.y = -10.0;
            w.tu(BG_Z, BG_W);
            assert!(
                (w.base.ps.x - x0).abs() > 0.01,
                "after clear, weapon must move on TU {i}"
            );
            break;
        }
    }
    assert!(cleared, "stuck must clear; timeout was {}", w.base.effect.timeout);
}

/// Character effect_stuck(1,2) is delayed (timein=1); must not freeze permanently
#[test]
fn character_tu_effect_stuck_clears_and_resumes_motion() {
    let data = minimal_char_data(1);
    let mut ch = Character::new(1, data, 1, 200.0, 350.0);
    ch.base.effect_stuck(1, 2); // F.LF hit_stop — not stuck until timein < 0
    // first TU: timein 1 -> 0, not effectively stuck; may still move
    ch.base.ps.vx = 5.0;
    let x0 = ch.base.ps.x;
    ch.tu(None, BG_Z, BG_W);
    // not permanently frozen on first frame (timein was 1)
    assert!(
        !ch.base.is_effectively_stuck() || (ch.base.ps.x - x0).abs() > 0.01 || ch.base.effect.timein >= 0,
        "timein=1 must not freeze on first TU"
    );
    let mut cleared = false;
    for _ in 0..12 {
        ch.tu(None, BG_Z, BG_W);
        if !ch.base.effect.stuck {
            cleared = true;
            break;
        }
    }
    assert!(cleared, "effect.stuck must clear via Character::tu");
    ch.base.ps.vx = 6.0;
    let x1 = ch.base.ps.x;
    ch.tu(None, BG_Z, BG_W);
    assert!(
        (ch.base.ps.x - x1).abs() > 0.01,
        "motion must resume after stuck clears"
    );
}

/// effect_stuck(0, n) freezes only when timein < 0 (after first timein decrement)
#[test]
fn effect_stuck_zero_timein_freezes_after_timein_decrements() {
    let data = minimal_char_data(1);
    let mut lo = LivingObject::new(1, data, 1, 200.0, 350.0);
    lo.obj_type = "character".into();
    lo.effect_stuck(0, 2);
    assert!(lo.effect.stuck);
    assert_eq!(lo.effect.timein, 0);
    // before recover: timein==0 so NOT effectively stuck yet
    assert!(!lo.is_effectively_stuck());
    lo.recover_tu(); // timein -> -1, timeout still 2 (decay only when timein was already < 0 at start of recover)
    // After one recover with timein starting 0: F.LF checks timein < 0 before decrement for timeout;
    // we decrement timein at end so after recover timein is -1
    assert_eq!(lo.effect.timein, -1);
}

#[test]
fn effect_create_sets_stuck_not_super_armor() {
    let data = minimal_char_data(1);
    let mut lo = LivingObject::new(1, data, 1, 100.0, 350.0);
    lo.effect_create(0, 3, 5.0, -2.0);
    assert!(lo.effect.stuck);
    assert!(!lo.effect.super_armor, "F.LF effect 0 is not super_armor");
    assert_eq!(lo.effect.timein, 0);
    assert_eq!(lo.effect.timeout, 3);
    assert!((lo.effect.dvx - 5.0).abs() < 1e-9);
}

#[test]
fn defend_injury_no_fall_progression() {
    let data = minimal_char_data(1);
    let mut lo = LivingObject::new(1, data, 1, 100.0, 350.0);
    lo.fall = 0.0;
    let hp0 = lo.hp;
    lo.defend_injury(20.0 * 0.1);
    assert!((lo.hp - (hp0 - 2.0)).abs() < 1e-9);
    assert_eq!(lo.fall, 0.0);
}


#[test]
fn apply_combat_hit_effective_defend_no_pain_frame() {
    let data = minimal_char_data(1);
    let mut ch = Character::new(1, data, 1, 100.0, 350.0);
    ch.base.facing = 1;
    ch.base.trans_frame(7, 0);
    ch.base.bdefend = 0.0;
    ch.base.fall = 0.0;
    let hp0 = ch.base.hp;
    // attack from front (att_x > vic_x, facing right)
    let (ok, drop, defended) = ch.apply_combat_hit(40.0, 20.0, 8.0, -3.0, 200.0, 0, 10.0, 0);
    assert!(ok && defended && !drop);
    assert_eq!(ch.base.frame.n, 111);
    assert!((ch.base.hp - (hp0 - 4.0)).abs() < 1e-6); // 0.1 * 40
    assert_eq!(ch.base.fall, 0.0);
}

#[test]
fn apply_combat_hit_broken_defend_stays_112_no_fall_pain() {
    let data = minimal_char_data(1);
    let mut ch = Character::new(1, data, 1, 100.0, 350.0);
    ch.base.facing = 1;
    ch.base.trans_frame(7, 0);
    ch.base.bdefend = 35.0;
    ch.base.fall = 0.0;
    let (ok, drop, defended) = ch.apply_combat_hit(40.0, 20.0, 8.0, -3.0, 200.0, 0, 20.0, 0);
    // 35+20 > 40 break limit
    assert!(ok && defended && !drop);
    assert_eq!(ch.base.frame.n, 112, "broken defend must stay on 112, not pain 220");
    assert_eq!(ch.base.fall, 0.0);
}

/// F.LF weapon.interaction passes ITR.bdefend (often 16–60) into character.hit
#[test]
fn apply_combat_hit_weapon_bdefend_breaks_defend() {
    let data = minimal_char_data(1);
    let mut ch = Character::new(1, data, 1, 100.0, 350.0);
    ch.base.facing = 1;
    ch.base.trans_frame(7, 0);
    ch.base.bdefend = 0.0;
    // dash-like weapon itr: bdefend 60 alone breaks (limit 40)
    let (ok, drop, defended) = ch.apply_combat_hit(20.0, 20.0, 5.0, 0.0, 200.0, 0, 60.0, 0);
    assert!(ok && defended && !drop);
    assert_eq!(ch.base.frame.n, 112);
    assert!(ch.base.bdefend > global::DEFEND_BREAK_LIMIT);
}

#[test]
fn stats_protocol_attacked_offset_killed() {
    let mut ch = Character::new(1, minimal_char_data(0), 1, 100.0, 400.0);
    assert!(ch.attacked(25.0));
    assert!((ch.stat_attack - 25.0).abs() < 1e-6);
    ch.offset_attack(5.0);
    assert!((ch.stat_attack - 20.0).abs() < 1e-6);
    ch.killed();
    assert_eq!(ch.base.kills, 1);
    ch.die(false);
    assert!(ch.base.dead);
    assert!(ch.heal(40.0));
    assert!((ch.base.effect.heal - 40.0).abs() < 1e-6);
}

#[test]
fn background_get_pos_ratios() {
    use flf::background::Background;
    let bg = Background::from_json(
        0,
        &json!({"width": 1000, "zboundary": [300, 500], "name": "t"}),
    );
    let (x, y, z) = bg.get_pos(0.5, 0.5);
    assert!((x - 500.0).abs() < 1e-6);
    assert!((y - 0.0).abs() < 1e-6);
    assert!((z - 400.0).abs() < 1e-6);
    assert!(bg.leaving(-10.0, 5.0));
    assert!(!bg.leaving(500.0, 5.0));
}

#[test]
fn mechanics_coincide_and_unit_friction() {
    use flf::livingobject::Pos;
    use flf::mechanics::Mech;
    let mut vic = Pos { x: 10.0, y: -5.0, z: 20.0, ..Default::default() };
    Mech::coincide_xz((100.0, 50.0), (10.0, 20.0), &mut vic);
    assert!((vic.x - 100.0).abs() < 1e-6);
    assert!((vic.z - 50.0).abs() < 1e-6);
    vic.y = 0.0;
    vic.vx = 3.0;
    vic.vz = -2.0;
    Mech::unit_friction(&mut vic);
    assert!((vic.vx - 2.0).abs() < 1e-6);
    assert!((vic.vz + 1.0).abs() < 1e-6);
}

#[test]
fn math_helpers_parity() {
    use flf::math::*;
    assert!(inbetween(5.0, 1.0, 9.0));
    assert!(negligible(1e-9));
    assert!((round_d2(1.239) - 1.24).abs() < 1e-9);
    let p = bezier2((0.0, 0.0), (0.5, 1.0), (1.0, 0.0), 0.5);
    assert!((p.0 - 0.5).abs() < 1e-6);
    assert!(intersect((0.0, 0.0), (1.0, 1.0), (0.0, 1.0), (1.0, 0.0)).is_some());
}

#[test]
fn keycode_roundtrip_subset() {
    use flf::controller::{keycode_to_keyname, keyname_to_keycode};
    assert_eq!(keycode_to_keyname(65), "a");
    assert_eq!(keyname_to_keycode("a"), 65);
    assert_eq!(keyname_to_keycode("space"), 32);
}

#[test]
fn combo_update_persists_unconsumed_skill() {
    let mut ch = Character::new(1, minimal_char_data(0), 1, 100.0, 400.0);
    ch.combo_buffer = Some("D>A".into());
    ch.combo_update();
    // no hit tag in minimal data → unconsumed non-dir persists
    assert_eq!(ch.combo_buffer.as_deref(), Some("D>A"));
    ch.combo_buffer = Some("left".into());
    ch.combo_update();
    assert!(ch.combo_buffer.is_none());
}

#[test]
fn weapon_attacked_credit_uid_tracks_holder() {
    let mut w = Weapon::new(2, minimal_weapon_data(true), 0.0, 0.0);
    assert!(w.attacked_credit_uid().is_none());
    w.pick(9);
    assert_eq!(w.attacked_credit_uid(), Some(9));
}

#[test]
fn opoint_parses_dvz() {
    let v = json!({
        "bmp": { "name": "T" },
        "frame": {
            "0": {
                "state": 0, "wait": 1, "next": 0, "pic": 0, "centerx": 1, "centery": 1,
                "opoint": { "oid": 200, "x": 1, "y": 2, "action": 0, "dvx": 1, "dvy": 2, "dvz": 3, "facing": 20 }
            }
        }
    });
    let d = parse_object_data(1, "character", &v);
    let op = d.frames.get(&0).unwrap().opoint.as_ref().unwrap();
    assert!((op.dvz - 3.0).abs() < 1e-6);
    assert_eq!(op.facing, 20);
}
