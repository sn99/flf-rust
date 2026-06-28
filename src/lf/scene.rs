//! Scene queries (LF/scene.js) — character lists + volume intersection
use crate::core_engine::collision::Volume;
use crate::lf::character::Character;
use crate::lf::mechanics::Mech;
use crate::lf::weapon::Weapon;

#[derive(Clone, Copy)]
pub struct QueryOpts {
    pub team: Option<i32>,
    pub not_team: Option<i32>,
    pub sort_distance: bool,
    pub reverse: bool,
}

/// Returns indices of characters matching filters, optionally sorted by distance to `from`
pub fn query_characters(
    chars: &[Character],
    from_idx: usize,
    opts: QueryOpts,
) -> Vec<usize> {
    let fx = chars.get(from_idx).map(|c| c.base.ps.x).unwrap_or(0.0);
    let fz = chars.get(from_idx).map(|c| c.base.ps.z).unwrap_or(0.0);
    let mut idx: Vec<usize> = (0..chars.len())
        .filter(|&i| {
            if i == from_idx || chars[i].base.removed {
                return false;
            }
            let t = chars[i].base.team;
            if let Some(team) = opts.team {
                if t != team {
                    return false;
                }
            }
            if let Some(nt) = opts.not_team {
                if t == nt {
                    return false;
                }
            }
            true
        })
        .collect();
    if opts.sort_distance {
        idx.sort_by(|&a, &b| {
            let da = (chars[a].base.ps.x - fx).hypot(chars[a].base.ps.z - fz);
            let db = (chars[b].base.ps.x - fx).hypot(chars[b].base.ps.z - fz);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    if opts.reverse {
        idx.reverse();
    }
    idx
}

/// Living weapon positions for AI / scripts
pub fn query_weapons(weapons: &[(u32, f64, f64, bool)]) -> Vec<(u32, f64, f64)> {
    weapons
        .iter()
        .filter(|(_, _, _, held)| !*held)
        .map(|(u, x, z, _)| (*u, *x, *z))
        .collect()
}

/// F.LF scene.query with body volumes — character indices whose bdy intersects `vol`
pub fn query_volume_characters(
    chars: &[Character],
    vol: &Volume,
    exclude_uid: Option<u32>,
    not_team: Option<i32>,
) -> Vec<usize> {
    let mut out = vec![];
    for (i, ch) in chars.iter().enumerate() {
        if ch.base.removed {
            continue;
        }
        if exclude_uid == Some(ch.base.uid) {
            continue;
        }
        if let Some(nt) = not_team {
            if ch.base.team == nt && ch.base.team != 0 {
                continue;
            }
        }
        let Some(fd) = ch.base.frame_data() else { continue };
        let bdys = Mech::body_volumes(&ch.base.ps, ch.base.facing, fd);
        if bdys.iter().any(|b| vol.intersects(b)) {
            out.push(i);
        }
    }
    out
}

/// Weapons whose body intersects volume (not held)
pub fn query_volume_weapons(
    weapons: &[Weapon],
    vol: &Volume,
    exclude_uid: Option<u32>,
) -> Vec<usize> {
    let mut out = vec![];
    for (i, w) in weapons.iter().enumerate() {
        if w.held || w.base.removed {
            continue;
        }
        if exclude_uid == Some(w.base.uid) {
            continue;
        }
        let Some(fd) = w.base.frame_data() else { continue };
        let bdys = Mech::body_volumes(&w.base.ps, w.base.facing, fd);
        if bdys.iter().any(|b| vol.intersects(b)) {
            out.push(i);
        }
    }
    out
}
