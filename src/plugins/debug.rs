//! Renders physics objects and events like [AABBs](ColliderAabb) and [contacts](Collision) for debugging purposes.
//!
//! See [`PhysicsDebugPlugin`].

use crate::prelude::*;
use bevy::prelude::*;

/// Renders physics objects and events like [AABBs](ColliderAabb) and [contacts](Collision) for debugging purposes.
///
/// You can configure what is rendered using the [`PhysicsDebugConfig`] resource.
pub struct PhysicsDebugPlugin;

impl Plugin for PhysicsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsDebugConfig>()
            .insert_resource(GizmoConfig {
                line_width: 1.0,
                ..default()
            })
            .register_type::<PhysicsDebugConfig>()
            // render AABBs first, so collider shapes drawn over the top. Looks better.
            .add_systems(
                PostUpdate,
                (
                    debug_render_aabbs
                        .run_if(|config: Res<PhysicsDebugConfig>| config.render_aabbs)
                        .after(PhysicsSet::Sync),
                    debug_render_colliders
                        .run_if(|config: Res<PhysicsDebugConfig>| config.render_colliders)
                        .after(PhysicsSet::Sync),
                    debug_render_contacts
                        .run_if(|config: Res<PhysicsDebugConfig>| config.render_contacts)
                        .after(PhysicsSet::Sync),
                ).chain()
            );
    }
}

/// Controls the [`PhysicsDebugPlugin`] configuration.
#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct PhysicsDebugConfig {
    /// Renders the Axis-Aligned Bounding Boxes of [colliders](`Collider`).
    pub render_aabbs: bool,
    /// Renders contact points.
    pub render_contacts: bool,
    /// Renders collider shapes
    pub render_colliders: bool,
}

impl Default for PhysicsDebugConfig {
    fn default() -> Self {
        Self {
            render_aabbs: true,
            render_contacts: true,
            render_colliders: true,
        }
    }
}

fn debug_render_colliders(cols: Query<(&Collider, &Transform)>, mut gizmos: Gizmos) {
    for (col, transform) in cols.iter() {
        let shape = col.get_shape();

        // render a "+" at center of collider
        //    a
        //    |
        //d -   - b
        //    |
        //    c
        let x_sz = 3.0; // length of arm of cross at center
        let a = transform.transform_point(Vec3::new(0.0, x_sz, 0.0)).truncate();
        let b = transform.transform_point(Vec3::new(x_sz, 0.0, 0.0)).truncate();
        let c = transform.transform_point(Vec3::new(0.0, -x_sz, 0.0)).truncate();
        let d = transform.transform_point(Vec3::new(-x_sz, 0.0, 0.0)).truncate();
        gizmos.line_2d(a, c, Color::GRAY);
        gizmos.line_2d(b, d, Color::GRAY);

        // render the collider shape

        if let Some(ball) = shape.as_ball() {
            gizmos.circle_2d(transform.translation.truncate(), ball.radius, Color::WHITE);
            continue;
        }

        if let Some(triangle) = shape.as_triangle() {
            let p1 = transform.transform_point(Vec3::new(triangle.a[0], triangle.a[1], 0.0)).truncate();
            let p2 = transform.transform_point(Vec3::new(triangle.b[0], triangle.b[1], 0.0)).truncate();
            let p3 = transform.transform_point(Vec3::new(triangle.c[0], triangle.c[1], 0.0)).truncate();
            gizmos.line_2d(p1, p2, Color::WHITE);
            gizmos.line_2d(p2, p3, Color::WHITE);
            gizmos.line_2d(p3, p1, Color::WHITE);
            continue;
        }
        
        if let Some(poly) = shape.as_convex_polygon() {
            let last_p = poly.points().last().unwrap();
            let mut start_p =  transform.transform_point(Vec3::new(last_p.x, last_p.y, 0.0)).truncate();
            for i in 0..poly.points().len() {
                let p = poly.points()[i];
                let tmp = transform.transform_point(Vec3::new(p.x, p.y, 0.0)).truncate();
                gizmos.line_2d(start_p, tmp, Color::WHITE);
                start_p = tmp;
            }
            continue;
        }

        if let Some(cuboid) = shape.as_cuboid() {
            let points: Vec<Vec3> = cuboid.to_polyline().into_iter().map(|p| Vec3::new(p.x, p.y, 0.0)).collect();
            let mut start_p = transform.transform_point(*points.last().unwrap());
            for i in 0..points.len() {
                let tmp = transform.transform_point(points[i]);
                gizmos.line_2d(start_p.truncate(), tmp.truncate(), Color::WHITE);
                start_p = tmp;
            }
            continue;
        }

        bevy::log::warn!("Can only render colliders for balls, cuboids, and polys at the mo.");
    }
}

fn debug_render_aabbs(aabbs: Query<&ColliderAabb>, mut gizmos: Gizmos) {
    #[cfg(feature = "2d")]
    for aabb in aabbs.iter() {
        gizmos.cuboid(
            Transform::from_scale(Vector::from(aabb.extents()).extend(0.0).as_f32())
                .with_translation(Vector::from(aabb.center()).extend(0.0).as_f32()),
            Color::GRAY,
        );
    }

    #[cfg(feature = "3d")]
    for aabb in aabbs.iter() {
        gizmos.cuboid(
            Transform::from_scale(Vector::from(aabb.extents()).as_f32())
                .with_translation(Vector::from(aabb.center()).as_f32()),
            Color::GRAY,
        );
    }
}

#[allow(clippy::unnecessary_cast)]
fn debug_render_contacts(mut collisions: EventReader<Collision>, mut gizmos: Gizmos) {
    #[cfg(feature = "2d")]
    for Collision(contact) in collisions.iter() {
        let p1 = contact.point1.as_f32();
        let p2 = contact.point2.as_f32();

        gizmos.line_2d(p1 - Vec2::X * 0.3, p1 + Vec2::X * 0.3, Color::CYAN);
        gizmos.line_2d(p1 - Vec2::Y * 0.3, p1 + Vec2::Y * 0.3, Color::CYAN);

        gizmos.line_2d(p2 - Vec2::X * 0.3, p2 + Vec2::X * 0.3, Color::CYAN);
        gizmos.line_2d(p2 - Vec2::Y * 0.3, p2 + Vec2::Y * 0.3, Color::CYAN);
    }
    #[cfg(feature = "3d")]
    for Collision(contact) in collisions.iter() {
        let p1 = contact.point1.as_f32();
        let p2 = contact.point2.as_f32();

        gizmos.line(p1 - Vec3::X * 0.3, p1 + Vec3::X * 0.3, Color::CYAN);
        gizmos.line(p1 - Vec3::Y * 0.3, p1 + Vec3::Y * 0.3, Color::CYAN);
        gizmos.line(p1 - Vec3::Z * 0.3, p1 + Vec3::Z * 0.3, Color::CYAN);

        gizmos.line(p2 - Vec3::X * 0.3, p2 + Vec3::X * 0.3, Color::CYAN);
        gizmos.line(p2 - Vec3::Y * 0.3, p2 + Vec3::Y * 0.3, Color::CYAN);
        gizmos.line(p2 - Vec3::Z * 0.3, p2 + Vec3::Z * 0.3, Color::CYAN);
    }
}
