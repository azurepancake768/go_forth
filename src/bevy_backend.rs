use crate::*;
use bevy::{prelude::*, window::PrimaryWindow};

pub fn run(level: Vec<TileStored>) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Level {
            width: 4,
            height: 4,
            level,
            side_moves: 3
        })
        .add_systems(Startup, setup)
        .add_systems(Update, handle_kb_input)
        .add_systems(PostUpdate, (render_positioned, render_level))
        .run();
}

fn render_positioned(
    mut q: Query<(&mut Transform, &Position)>,
    w: Query<&Window, With<PrimaryWindow>>,
    level: Res<Level>,
) {
    let window = w.single();

    for (
        mut t,
        Position {
            position: p,
            rotation: rot,
        },
    ) in q.iter_mut()
    {
        *t = Transform {
            translation: level_to_world_pos(p.x as u8, p.y as u8, &level, window).extend(0.0),
            rotation: rot.unwrap_or(Facing::Up).rotation_quat(),
            ..*t
        };
    }
}

fn render_level(
    mut q: Query<(&mut Handle<Image>, &mut TileComponent, &Position)>,
    assets: Res<AssetServer>,
    level: Res<Level>,
) {
    for (mut tex, mut tile, &Position { position: p, .. }) in q.iter_mut() {
        tile.0 = level.tile_at_vec(p);
        *tex = assets.load(format!("sprites/tiles/{}.png", tile.0.name()));
    }

}

fn setup(
    mut commands: Commands,
    q: Query<&Window, With<PrimaryWindow>>,
    assets: Res<AssetServer>,
    level: Res<Level>,
) {
    let window = q.get_single().expect("Only one primary window");

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
        ..default()
    });

    commands.spawn(TextBundle{text: Text::from_section("yo momma so very fat", TextStyle {
        font_size: 18.0, color: Color::Rgba { red: 255.0, green: 255.0, blue: 255.0, alpha: 255.0 }, ..default()}), ..default()});

    for i in 0..level.width {
        for j in 0..level.height {
            let (i, j) = (i as u8, j as u8);
            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_scale(Vec3::splat(TILE_TEXTURE_SCALE)),
                    texture: assets
                        .load(format!("sprites/tiles/{}.png", level.tile_at(i, j).name())),
                    ..default()
                },
                Position::pos(i, j),
                TileComponent(level.tile_at(i, j)),
            ));
        }
    }

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(TILE_TEXTURE_SCALE)),
            texture: assets.load("sprites/player.png"),
            ..default()
        },
        Position::new(0, 0, Some(Facing::Up)),
        Player,
    ));
}

fn handle_kb_input(
    mut q: Query<&mut Position, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    level: ResMut<Level>,
) {
    let level = level.into_inner();
    let pos = q.single_mut();
    let transform  = pos.into_inner();
    match match keyboard_input.get_just_pressed().nth(0) {
        Some(KeyCode::Up) => Some(Facing::Up),
        Some(KeyCode::W)  => Some(Facing::Up),
        Some(KeyCode::Down) => Some(Facing::Down),
            Some(KeyCode::S)    => Some(Facing::Down),
        Some(KeyCode::Right) => Some(Facing::Right),
        Some(KeyCode::D)     => Some(Facing::Right),
        Some(KeyCode::Left)  => Some(Facing::Left),
        Some(KeyCode::A)     => Some(Facing::Left),
        _ => None
    } {
        Some(dir) => {
            move_player(level, transform, dir);
        }
        None => {}
    };
}
#[derive(Component)]
struct TileComponent(TileStored);
