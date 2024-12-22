use bevy::prelude::*;

#[derive(Component)]
pub struct Rotate;

pub fn rotate(mut query: Query<&mut Transform, With<Rotate>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}
