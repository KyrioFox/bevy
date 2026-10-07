use bevy::prelude::*;

const ARENA_HALF_WIDTH: f32 = 560.0;
const GROUND_Y: f32 = -230.0;
const FIGHTER_HALF_HEIGHT: f32 = 58.0;
const WALK_SPEED: f32 = 330.0;
const GRAVITY: f32 = 1_250.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dragon Ball Arena".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<MatchState>()
        .init_resource::<DamageQueue>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                read_player_input,
                drive_cpu,
                move_fighters,
                fighter_attacks,
                move_and_collide_projectiles,
                apply_queued_damage,
                advance_round,
            )
                .chain(),
        )
        .add_systems(Update, (sync_fighter_auras, refresh_hud).chain())
        .run();
}

#[derive(Component)]
struct PlayerFighter;

#[derive(Component)]
struct CpuFighter;

#[derive(Component)]
struct MatchHud;

#[derive(Component)]
struct FighterAura(Entity);

#[derive(Component)]
struct Fighter {
    name: &'static str,
    health: u16,
    ki: u16,
    ki_regen: f32,
    velocity: Vec2,
    facing: f32,
    intent: FighterIntent,
    light_cooldown: f32,
    blast_cooldown: f32,
    dash_cooldown: f32,
    dash_time: f32,
    hit_flash: f32,
}

impl Fighter {
    fn new(name: &'static str, facing: f32) -> Self {
        Self {
            name,
            health: 100,
            ki: 100,
            ki_regen: 0.0,
            velocity: Vec2::ZERO,
            facing,
            intent: FighterIntent::default(),
            light_cooldown: 0.0,
            blast_cooldown: 0.0,
            dash_cooldown: 0.0,
            dash_time: 0.0,
            hit_flash: 0.0,
        }
    }
}

#[derive(Default)]
struct FighterIntent {
    movement: f32,
    jump: bool,
    strike: bool,
    blast: bool,
    dash: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Player,
    Cpu,
}

#[derive(Component)]
struct KiBlast {
    owner: Entity,
    side: Side,
    velocity: Vec2,
    lifetime: f32,
}

#[derive(Resource, Default)]
struct DamageQueue(Vec<(Entity, u16)>);

#[derive(Resource)]
struct MatchState {
    round: u32,
    player_wins: u32,
    cpu_wins: u32,
    resolved: bool,
    reset_in: f32,
    message: &'static str,
}

impl Default for MatchState {
    fn default() -> Self {
        Self {
            round: 1,
            player_wins: 0,
            cpu_wins: 0,
            resolved: false,
            reset_in: 0.0,
            message: "Fight!",
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1_280.0, 720.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.035, 0.055, 0.12))),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.12, 0.18, 0.27), Vec2::new(1_280.0, 24.0)),
        Transform::from_xyz(0.0, GROUND_Y - 22.0, -1.0),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.96, 0.62, 0.12), Vec2::new(1_280.0, 4.0)),
        Transform::from_xyz(0.0, GROUND_Y - 8.0, 0.0),
    ));

    spawn_fighter(
        &mut commands,
        Side::Player,
        Vec2::new(-300.0, GROUND_Y + FIGHTER_HALF_HEIGHT),
    );
    spawn_fighter(
        &mut commands,
        Side::Cpu,
        Vec2::new(300.0, GROUND_Y + FIGHTER_HALF_HEIGHT),
    );

    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.97, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: px(18),
            left: px(22),
            ..default()
        },
        MatchHud,
    ));
    commands.spawn((
        Text::new("A / D move    SPACE jump    J strike    K ki blast    LEFT SHIFT dash"),
        TextFont {
            font_size: FontSize::Px(17.0),
            ..default()
        },
        TextColor(Color::srgb(0.72, 0.78, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(18),
            left: px(22),
            ..default()
        },
    ));
}

fn spawn_fighter(commands: &mut Commands, side: Side, position: Vec2) {
    let (name, facing, body, aura) = match side {
        Side::Player => (
            "Goku",
            1.0,
            Color::srgb(0.96, 0.38, 0.08),
            Color::srgba(1.0, 0.5, 0.08, 0.18),
        ),
        Side::Cpu => (
            "Vegeta",
            -1.0,
            Color::srgb(0.12, 0.38, 0.96),
            Color::srgba(0.18, 0.46, 1.0, 0.18),
        ),
    };
    let entity = commands
        .spawn((
            Sprite::from_color(body, Vec2::new(64.0, 116.0)),
            Transform::from_xyz(position.x, position.y, 1.0),
            Fighter::new(name, facing),
        ))
        .id();
    match side {
        Side::Player => {
            commands.entity(entity).insert(PlayerFighter);
        }
        Side::Cpu => {
            commands.entity(entity).insert(CpuFighter);
        }
    }
    commands.spawn((
        Sprite::from_color(aura, Vec2::new(112.0, 150.0)),
        Transform::from_xyz(position.x, position.y, 0.2),
        FighterAura(entity),
    ));
}

fn read_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<MatchState>,
    mut player: Single<&mut Fighter, With<PlayerFighter>>,
) {
    let fighter = &mut *player;
    fighter.intent = FighterIntent::default();
    if state.resolved {
        return;
    }
    fighter.intent.movement = (if keyboard.pressed(KeyCode::KeyD) {
        1.0
    } else {
        0.0
    }) - (if keyboard.pressed(KeyCode::KeyA) {
        1.0
    } else {
        0.0
    });
    fighter.intent.jump = keyboard.just_pressed(KeyCode::Space);
    fighter.intent.strike = keyboard.just_pressed(KeyCode::KeyJ);
    fighter.intent.blast = keyboard.just_pressed(KeyCode::KeyK);
    fighter.intent.dash = keyboard.just_pressed(KeyCode::ShiftLeft);
}

fn drive_cpu(
    state: Res<MatchState>,
    player: Single<&Transform, With<PlayerFighter>>,
    mut cpu: Single<(&mut Fighter, &Transform), With<CpuFighter>>,
) {
    let (fighter, transform) = &mut *cpu;
    fighter.intent = FighterIntent::default();
    if state.resolved {
        return;
    }
    let distance = player.translation.x - transform.translation.x;
    fighter.intent.movement = if distance.abs() > 125.0 {
        distance.signum()
    } else {
        0.0
    };
    fighter.intent.strike = distance.abs() <= 125.0 && fighter.light_cooldown <= 0.0;
    fighter.intent.blast =
        distance.abs() > 245.0 && fighter.ki >= 18 && fighter.blast_cooldown <= 0.0;
    fighter.intent.dash = distance.abs() > 310.0 && fighter.dash_cooldown <= 0.0;
}

fn move_fighters(
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    mut fighters: Query<(&mut Fighter, &mut Transform, &mut Sprite)>,
) {
    if state.resolved {
        return;
    }
    let delta = time.delta_secs();
    for (mut fighter, mut transform, mut sprite) in &mut fighters {
        fighter.light_cooldown = (fighter.light_cooldown - delta).max(0.0);
        fighter.blast_cooldown = (fighter.blast_cooldown - delta).max(0.0);
        fighter.dash_cooldown = (fighter.dash_cooldown - delta).max(0.0);
        fighter.dash_time = (fighter.dash_time - delta).max(0.0);
        fighter.hit_flash = (fighter.hit_flash - delta).max(0.0);
        if fighter.intent.movement != 0.0 {
            fighter.facing = fighter.intent.movement.signum();
        }
        if fighter.intent.dash && fighter.dash_cooldown <= 0.0 {
            fighter.dash_time = 0.16;
            fighter.dash_cooldown = 1.15;
        }
        let speed = if fighter.dash_time > 0.0 {
            820.0
        } else {
            WALK_SPEED
        };
        fighter.velocity.x = if fighter.dash_time > 0.0 {
            fighter.facing * speed
        } else {
            fighter.intent.movement * speed
        };
        if fighter.intent.jump && transform.translation.y <= GROUND_Y + FIGHTER_HALF_HEIGHT + 1.0 {
            fighter.velocity.y = 520.0;
        }
        fighter.velocity.y -= GRAVITY * delta;
        transform.translation.x = (transform.translation.x + fighter.velocity.x * delta)
            .clamp(-ARENA_HALF_WIDTH + 32.0, ARENA_HALF_WIDTH - 32.0);
        transform.translation.y += fighter.velocity.y * delta;
        if transform.translation.y < GROUND_Y + FIGHTER_HALF_HEIGHT {
            transform.translation.y = GROUND_Y + FIGHTER_HALF_HEIGHT;
            fighter.velocity.y = 0.0;
        }
        sprite.color = match fighter.hit_flash > 0.0 {
            true => Color::WHITE,
            false if fighter.name == "Goku" => Color::srgb(0.96, 0.38, 0.08),
            false => Color::srgb(0.12, 0.38, 0.96),
        };
    }
}

fn fighter_attacks(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    mut damage: ResMut<DamageQueue>,
    mut fighters: Query<(Entity, &mut Fighter, &Transform)>,
) {
    if state.resolved {
        return;
    }
    let positions: Vec<_> = fighters
        .iter()
        .map(|(entity, fighter, transform)| (entity, fighter.name, transform.translation))
        .collect();
    for (entity, mut fighter, transform) in &mut fighters {
        if fighter.intent.strike && fighter.light_cooldown <= 0.0 {
            fighter.light_cooldown = 0.42;
            let target = positions.iter().find(|(target, _, position)| {
                *target != entity
                    && (position.x - transform.translation.x).abs() < 118.0
                    && (position.y - transform.translation.y).abs() < 96.0
                    && (position.x - transform.translation.x).signum() == fighter.facing
            });
            if let Some((target, _, _)) = target {
                damage.0.push((*target, 12));
            }
        }
        if fighter.intent.blast && fighter.blast_cooldown <= 0.0 && fighter.ki >= 18 {
            fighter.blast_cooldown = 0.72;
            fighter.ki -= 18;
            let side = if fighter.name == "Goku" {
                Side::Player
            } else {
                Side::Cpu
            };
            commands.spawn((
                Sprite::from_color(
                    if side == Side::Player {
                        Color::srgb(1.0, 0.78, 0.18)
                    } else {
                        Color::srgb(0.45, 0.7, 1.0)
                    },
                    Vec2::splat(30.0),
                ),
                Transform::from_xyz(
                    transform.translation.x + fighter.facing * 50.0,
                    transform.translation.y + 8.0,
                    2.0,
                ),
                KiBlast {
                    owner: entity,
                    side,
                    velocity: Vec2::new(fighter.facing * 590.0, 0.0),
                    lifetime: 1.8,
                },
            ));
        }
        fighter.ki_regen += time.delta_secs() * 5.0;
        let regenerated = fighter.ki_regen.floor() as u16;
        fighter.ki = (fighter.ki + regenerated).min(100);
        fighter.ki_regen = if fighter.ki == 100 {
            0.0
        } else {
            fighter.ki_regen - f32::from(regenerated)
        };
    }
}

fn move_and_collide_projectiles(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<MatchState>,
    mut blasts: Query<(Entity, &mut Transform, &mut KiBlast), Without<Fighter>>,
    mut fighters: Query<(Entity, &Transform, &mut Fighter), Without<KiBlast>>,
) {
    let delta = time.delta_secs();
    for (blast_entity, mut blast_transform, mut blast) in &mut blasts {
        if state.resolved {
            commands.entity(blast_entity).despawn();
            continue;
        }
        blast_transform.translation.x += blast.velocity.x * delta;
        blast.lifetime -= delta;
        let target = fighters
            .iter_mut()
            .find_map(|(fighter_entity, transform, mut fighter)| {
                if fighter_entity != blast.owner
                    && ((transform.translation.x - blast_transform.translation.x).abs() < 44.0)
                    && ((transform.translation.y - blast_transform.translation.y).abs() < 66.0)
                {
                    let target_side = if fighter.name == "Goku" {
                        Side::Player
                    } else {
                        Side::Cpu
                    };
                    if target_side != blast.side {
                        fighter.health = fighter.health.saturating_sub(10);
                        fighter.hit_flash = 0.12;
                        return Some(fighter_entity);
                    }
                }
                None
            });
        if target.is_some()
            || blast.lifetime <= 0.0
            || blast_transform.translation.x.abs() > ARENA_HALF_WIDTH
        {
            commands.entity(blast_entity).despawn();
        }
    }
}

fn apply_queued_damage(
    mut damage: ResMut<DamageQueue>,
    mut fighters: Query<(&mut Fighter, &mut Sprite)>,
) {
    for (entity, amount) in damage.0.drain(..) {
        if let Ok((mut fighter, mut sprite)) = fighters.get_mut(entity) {
            fighter.health = fighter.health.saturating_sub(amount);
            fighter.hit_flash = 0.12;
            sprite.color = Color::WHITE;
        }
    }
}

fn advance_round(
    time: Res<Time<Fixed>>,
    mut state: ResMut<MatchState>,
    mut fighters: Query<(&mut Fighter, &mut Transform, &mut Sprite)>,
) {
    let player_down = fighters
        .iter()
        .any(|(fighter, _, _)| fighter.name == "Goku" && fighter.health == 0);
    let cpu_down = fighters
        .iter()
        .any(|(fighter, _, _)| fighter.name == "Vegeta" && fighter.health == 0);
    if !state.resolved && (player_down || cpu_down) {
        state.resolved = true;
        state.reset_in = 2.0;
        if player_down {
            state.cpu_wins += 1;
            state.message = "Vegeta wins the round";
        } else {
            state.player_wins += 1;
            state.message = "Goku wins the round";
        }
    }
    if state.resolved {
        state.reset_in -= time.delta_secs();
        if state.reset_in <= 0.0 {
            state.round += 1;
            state.resolved = false;
            state.message = "Fight!";
            for (mut fighter, mut transform, mut sprite) in &mut fighters {
                fighter.health = 100;
                fighter.ki = 100;
                fighter.ki_regen = 0.0;
                fighter.velocity = Vec2::ZERO;
                fighter.hit_flash = 0.0;
                fighter.light_cooldown = 0.0;
                fighter.blast_cooldown = 0.0;
                fighter.dash_cooldown = 0.0;
                transform.translation.x = if fighter.name == "Goku" {
                    -300.0
                } else {
                    300.0
                };
                transform.translation.y = GROUND_Y + FIGHTER_HALF_HEIGHT;
                sprite.color = if fighter.name == "Goku" {
                    Color::srgb(0.96, 0.38, 0.08)
                } else {
                    Color::srgb(0.12, 0.38, 0.96)
                };
            }
        }
    }
}

fn refresh_hud(
    state: Res<MatchState>,
    fighters: Query<&Fighter>,
    mut hud: Single<&mut Text, With<MatchHud>>,
) {
    let player = fighters.iter().find(|fighter| fighter.name == "Goku");
    let cpu = fighters.iter().find(|fighter| fighter.name == "Vegeta");
    if let (Some(player), Some(cpu)) = (player, cpu) {
        hud.0 = format!(
            "ROUND {}      {}  HP {:>3}  KI {:>3}      {:^24}      {}  HP {:>3}  KI {:>3}      SCORE {} - {}",
            state.round,
            player.name,
            player.health,
            player.ki,
            state.message,
            cpu.name,
            cpu.health,
            cpu.ki,
            state.player_wins,
            state.cpu_wins,
        );
    }
}

fn sync_fighter_auras(
    fighters: Query<&Transform, With<Fighter>>,
    mut auras: Query<(&mut Transform, &FighterAura), Without<Fighter>>,
) {
    for (mut aura_transform, aura) in &mut auras {
        if let Ok(fighter_transform) = fighters.get(aura.0) {
            aura_transform.translation.x = fighter_transform.translation.x;
            aura_transform.translation.y = fighter_transform.translation.y;
        }
    }
}
