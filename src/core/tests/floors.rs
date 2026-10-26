//! The maps' floors, checked in every level.

use super::common::started_room;
use crate::core::objects::ids::ObjectId;
use crate::core::rooms::LEVEL_ROOMS;

/// The floor under a player is found at whole pixels (position_meeting, as GameMaker's
/// runner checks it: a point at or below the box's top). A top a little below a whole
/// pixel (960.07) is found one pixel lower, and whoever stands on it sinks a pixel into
/// what is drawn there, as on the rock in Hide and Seek 2 the original room had at 960.0663.
#[test]
fn no_floor_top_lies_just_below_a_whole_pixel() {
    for room in LEVEL_ROOMS {
        let world = started_room(room);
        for id in world.ids_of(ObjectId::SolidParent) {
            let (instance, Some(bbox)) = (&world.instances[id], world.bbox(id)) else { continue };
            let below = bbox.top - bbox.top.floor();
            // Rotated blocks are slopes, whose edges are not meant to be on a pixel.
            let sinks = instance.image_angle == 0.0 && (0.001..0.5).contains(&below);
            assert!(!sinks, "{room:?}: the top of {:?} at ({}, {}) is {}", instance.object, instance.x, instance.y, bbox.top);
        }
    }
}
