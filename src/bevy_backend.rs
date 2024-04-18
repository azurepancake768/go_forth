use crate::*;
use bevy::{prelude::*, window::PrimaryWindow, asset::{AssetLoader, io::Reader, LoadContext}, utils::thiserror, input::keyboard::{KeyboardInput, Key}};

pub fn run(level: Level) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(level)
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
            rotation: rot.rotation_quat(),
            ..*t
        };
    }
}

fn render_level(
    mut q_tiles: Query<(&mut Handle<Image>, &mut TileComponent, &Position)>,
    mut q_text: Query<&mut Text, With<SideMovesText>>,
    assets: Res<AssetServer>,
    level: Res<Level>,
) {
    for (mut tex, mut tile, &Position { position: p, .. }) in q_tiles.iter_mut() {
        let tile = &mut tile.0;
        *tile = level.tile_at_vec(p);
        *tex = assets.load(format!("sprites/tiles/{}.png", tile.name()));
    }
    q_text.single_mut().sections[0].value = format!("{} side moves left", level.side_moves);
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

    commands.spawn((TextBundle {
            text: Text::from_section(format!("{} side moves left", level.side_moves), TextStyle {
                font_size: 18.0,
                color: Color::Rgba { red: 255.0, green: 255.0, blue: 255.0, alpha: 255.0 },
                ..default()
        }),
        ..default()}, SideMovesText));

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
        Position::new(0, 0, Facing::Up),
        Player,
    ));
}

fn handle_kb_input(
    mut q: Query<&mut Position, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    level: ResMut<Level>,
) {
    let level = level.into_inner();
    let transform = q.single_mut().into_inner();
    let pressed_keys = keyboard_input.into_inner().get_just_pressed().collect::<Vec<&KeyCode>>();
    if pressed_keys.len() > 0{
    match match pressed_keys[0]{
        KeyCode::ArrowUp => Some(Facing::Up),
        KeyCode::KeyW  => Some(Facing::Up),
        KeyCode::ArrowDown => Some(Facing::Down),
        KeyCode::KeyS    => Some(Facing::Down),
        KeyCode::ArrowRight => Some(Facing::Right),
        KeyCode::KeyD     => Some(Facing::Right),
        KeyCode::ArrowLeft  => Some(Facing::Left),
        KeyCode::KeyA     => Some(Facing::Left),
        _ => None
    } {
        Some(dir) => {
            move_player(level, transform, dir);
        }
        None => {}
    };
    };
}

#[derive(Component)]
struct TileComponent(TileStored);

#[derive(Component)]
struct SideMovesText;
