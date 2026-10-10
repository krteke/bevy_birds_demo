use super::*;

const FIELD: Vec2 = Vec2::new(640.0, 400.0);

fn flock(birds: Vec<Bird>) -> Flock {
    ComputeTaskPool::get_or_init(|| bevy::tasks::TaskPoolBuilder::new().num_threads(4).build());
    let mut flock = Flock { birds, ..default() };
    flock.grid.rebuild(&flock.birds, NEIGHBOR_RADIUS);
    flock
}

#[test]
fn grid_matches_distance_search_across_negative_cell_boundaries() {
    let flock = flock(
        (-8..8)
            .flat_map(|x| {
                (-8..8).map(move |y| {
                    Bird::new(
                        Vec2::new(x as f32 * 19.0 - 0.5, y as f32 * 23.0),
                        Vec2::X * 100.0,
                    )
                })
            })
            .collect(),
    );
    for index in 0..flock.birds.len() {
        let mut actual = Vec::new();
        flock
            .grid
            .for_each_neighbor(&flock.birds, index, |other, _, _| actual.push(other));
        actual.sort_unstable();
        let expected: Vec<_> = flock
            .birds
            .iter()
            .enumerate()
            .filter(|(other, bird)| {
                *other != index
                    && bird.position.distance_squared(flock.birds[index].position)
                        <= NEIGHBOR_RADIUS * NEIGHBOR_RADIUS
            })
            .map(|(other, _)| other)
            .collect();
        assert_eq!(actual, expected, "neighbors of bird {index}");
    }
}

#[test]
fn close_neighbors_steer_apart() {
    let flock = flock(vec![
        Bird::new(Vec2::new(-4.0, 0.0), Vec2::Y * 100.0),
        Bird::new(Vec2::new(4.0, 0.0), Vec2::Y * 100.0),
    ]);
    assert!(
        flock
            .flight_forces(
                0,
                Vec2::new(-4.0, 0.0),
                FIELD,
                &MouseField::default(),
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .x
            < 0.0
    );
    assert!(
        flock
            .flight_forces(
                1,
                Vec2::new(4.0, 0.0),
                FIELD,
                &MouseField::default(),
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .x
            > 0.0
    );
}

#[test]
fn birds_turn_toward_neighbor_heading() {
    let flock = flock(vec![
        Bird::new(Vec2::ZERO, Vec2::X * 100.0),
        Bird::new(Vec2::new(40.0, 0.0), Vec2::Y * 100.0),
    ]);
    assert!(
        flock
            .flight_forces(
                0,
                Vec2::ZERO,
                FIELD,
                &MouseField::default(),
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .y
            > 0.0
    );
}

#[test]
fn birds_turn_toward_neighbor_center() {
    let flock = flock(vec![
        Bird::new(Vec2::ZERO, Vec2::Y * 100.0),
        Bird::new(Vec2::new(40.0, 0.0), Vec2::Y * 100.0),
    ]);
    assert!(
        flock
            .flight_forces(
                0,
                Vec2::ZERO,
                FIELD,
                &MouseField::default(),
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .x
            > 0.0
    );
}

#[test]
fn separated_groups_turn_back_toward_flock_center() {
    let flock = flock(vec![
        Bird::new(Vec2::new(-200.0, 0.0), Vec2::Y * 100.0),
        Bird::new(Vec2::new(200.0, 0.0), Vec2::Y * 100.0),
    ]);
    assert!(
        flock
            .flight_forces(
                0,
                Vec2::ZERO,
                FIELD,
                &MouseField::default(),
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .x
            > 0.0
    );
    assert!(
        flock
            .flight_forces(
                1,
                Vec2::ZERO,
                FIELD,
                &MouseField::default(),
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .x
            < 0.0
    );
}

#[test]
fn cursor_attracts_birds() {
    let flock = flock(vec![Bird::new(Vec2::ZERO, Vec2::Y * 100.0)]);
    let mouse = MouseField {
        cursor: Some(Vec2::new(200.0, 0.0)),
        ..default()
    };
    assert!(
        flock
            .flight_forces(
                0,
                Vec2::ZERO,
                FIELD,
                &mouse,
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .x
            > 0.0
    );
}

#[test]
fn predator_cursor_repels_birds() {
    let flock = flock(vec![Bird::new(Vec2::new(20.0, 0.0), Vec2::Y * 100.0)]);
    let mouse = MouseField {
        cursor: Some(Vec2::ZERO),
        repelling: true,
        ..default()
    };
    assert!(
        flock
            .flight_forces(
                0,
                Vec2::new(20.0, 0.0),
                FIELD,
                &mouse,
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .x
            > 0.0
    );
}

#[test]
fn fast_predator_movement_repels_birds_between_cursor_endpoints() {
    let flock = flock(vec![Bird::new(Vec2::new(0.0, 20.0), Vec2::X * 100.0)]);
    let mut mouse = MouseField::default();
    mouse.update(Some(Vec2::new(-300.0, 0.0)), true, 1.0 / 60.0);
    mouse.update(Some(Vec2::new(300.0, 0.0)), true, 1.0 / 60.0);
    mouse.advance(1.0 / 60.0);
    assert!(
        flock
            .flight_forces(
                0,
                Vec2::new(0.0, 20.0),
                FIELD,
                &mouse,
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .y
            > 0.0
    );
}

#[test]
fn switching_to_attraction_clears_pending_predator_paths() {
    let flock = flock(vec![Bird::new(Vec2::new(0.0, 20.0), Vec2::X * 100.0)]);
    let mut mouse = MouseField::default();
    mouse.update(Some(Vec2::new(-300.0, 0.0)), true, 1.0 / 60.0);
    mouse.update(Some(Vec2::new(300.0, 0.0)), true, 1.0 / 60.0);
    mouse.update(Some(Vec2::new(300.0, 0.0)), false, 1.0 / 60.0);
    assert!(mouse.segments.is_empty() && mouse.pending.is_empty());
    let acceleration = flock
        .flight_forces(
            0,
            Vec2::new(0.0, 20.0),
            FIELD,
            &mouse,
            &SimulationSettings::default(),
            &[],
        )
        .steering;
    assert!(acceleration.x > 0.0 && acceleration.y < 0.0);
    mouse.update(Some(Vec2::new(400.0, 0.0)), false, 1.0 / 60.0);
    mouse.update(Some(Vec2::new(500.0, 0.0)), true, 1.0 / 60.0);
    assert!(
        mouse.segments.is_empty(),
        "attraction must not become a cut"
    );
}

#[test]
fn disabling_mouse_clears_old_cuts() {
    let mut mouse = MouseField::default();
    mouse.update(Some(Vec2::new(-300.0, 0.0)), true, 1.0 / 60.0);
    mouse.update(Some(Vec2::new(300.0, 0.0)), true, 1.0 / 60.0);
    assert!(mouse.cursor.is_some() && !mouse.pending.is_empty());
    mouse.update(None, false, 1.0 / 60.0);
    assert!(mouse.cursor.is_none());
    assert!(mouse.segments.is_empty() && mouse.pending.is_empty());
    mouse.update(Some(Vec2::new(400.0, 0.0)), true, 1.0 / 60.0);
    assert!(
        mouse.segments.is_empty(),
        "reentry must not connect to an old cursor position"
    );
}

#[test]
fn a_cut_is_consumed_by_one_simulation_tick() {
    let mut flock = flock(vec![Bird::new(Vec2::ZERO, Vec2::X * 100.0)]);
    let mut mouse = MouseField::default();
    mouse.update(Some(Vec2::new(-300.0, 0.0)), true, 1.0 / 60.0);
    mouse.update(Some(Vec2::new(300.0, 0.0)), true, 1.0 / 60.0);
    assert!(!mouse.pending.is_empty());
    flock.step(
        1.0 / 60.0,
        FIELD,
        &mut mouse,
        &SimulationSettings::default(),
        &[],
    );
    assert!(mouse.segments.is_empty() && mouse.pending.is_empty());
}

#[test]
fn population_changes_preserve_existing_birds() {
    let initial = Bird::new(Vec2::new(30.0, 40.0), Vec2::X * 120.0);
    let mut flock = flock(vec![initial.clone()]);
    flock.set_count(100, FIELD, SimulationSettings::default().cruise_speed);
    assert_eq!(flock.birds.len(), 100);
    assert_eq!(flock.birds[0], initial);
    flock.set_count(50, FIELD, SimulationSettings::default().cruise_speed);
    assert_eq!(flock.birds.len(), 50);
    assert_eq!(flock.birds[0], initial);
}

#[test]
fn coincident_birds_separate_without_nonfinite_values() {
    let mut flock = flock(vec![
        Bird::new(Vec2::ZERO, Vec2::X * 100.0),
        Bird::new(Vec2::ZERO, Vec2::X * 100.0),
    ]);
    flock.step(
        1.0 / 60.0,
        FIELD,
        &mut MouseField::default(),
        &SimulationSettings::default(),
        &[],
    );
    assert!(flock.birds[0].position.distance(flock.birds[1].position) > 0.0);
    for bird in &flock.birds {
        assert!(bird.position.is_finite());
        assert!(bird.velocity.is_finite());
    }
}

#[test]
fn long_simulation_keeps_speed_bounded() {
    let mut flock = flock(vec![Bird::new(Vec2::ZERO, Vec2::X * 100.0)]);
    let mut mouse = MouseField {
        cursor: Some(Vec2::new(200.0, 0.0)),
        ..default()
    };
    for _ in 0..600 {
        flock.step(
            1.0 / 60.0,
            FIELD,
            &mut mouse,
            &SimulationSettings::default(),
            &[],
        );
    }
    assert!(
        flock.birds[0].position != Vec2::ZERO,
        "simulation must advance"
    );
    for bird in &flock.birds {
        assert!(bird.velocity.is_finite());
        assert!(bird.velocity.length() <= MAX_SPEED + 0.001);
    }
}

#[test]
fn approaching_edge_steers_inward() {
    let flock = flock(vec![Bird::new(Vec2::new(620.0, 0.0), Vec2::X * 150.0)]);
    assert!(
        flock
            .flight_forces(
                0,
                Vec2::new(620.0, 0.0),
                FIELD,
                &MouseField::default(),
                &SimulationSettings::default(),
                &[]
            )
            .steering
            .x
            < 0.0
    );
}

#[test]
fn ordinary_edge_crossing_preserves_motion_history() {
    let mut flock = flock(vec![Bird::new(Vec2::new(641.0, 0.0), Vec2::X * 135.0)]);
    flock.step(
        1.0 / 60.0,
        FIELD,
        &mut MouseField::default(),
        &SimulationSettings::default(),
        &[],
    );
    let bird = &flock.birds[0];
    assert_eq!(bird.previous_position, Vec2::new(641.0, 0.0));
    assert_eq!(bird.trail[0], Vec2::new(641.0, 0.0));
    assert!(bird.velocity.x > 0.0, "edges should turn birds smoothly");
}

#[test]
fn resize_brings_outside_birds_into_the_visible_field_without_a_long_trail() {
    let mut flock = flock(vec![Bird::new(Vec2::new(600.0, 350.0), Vec2::X * 100.0)]);
    flock.resize_field(Vec2::new(200.0, 150.0), &[]);
    flock.step(
        1.0 / 60.0,
        Vec2::new(200.0, 150.0),
        &mut MouseField::default(),
        &SimulationSettings::default(),
        &[],
    );
    let bird = &flock.birds[0];
    assert!(bird.position.x.abs() <= 200.0 && bird.position.y.abs() <= 150.0);
    assert!(
        bird.trail
            .iter()
            .all(|point| point.x.abs() <= 200.0 && point.y.abs() <= 150.0)
    );
}

#[test]
fn trail_records_motion_on_fixed_intervals() {
    let mut flock = flock(vec![Bird::new(Vec2::ZERO, Vec2::X * 100.0)]);
    for _ in 0..4 {
        flock.step(
            1.0 / 60.0,
            FIELD,
            &mut MouseField::default(),
            &SimulationSettings::default(),
            &[],
        );
    }
    assert!(flock.birds[0].trail.iter().any(|point| point.x > 0.0));
    assert!(flock.birds[0].trail.contains(&Vec2::ZERO));
}

#[test]
fn a_dense_flock_keeps_flying_instead_of_settling_into_stationary_particles() {
    let mut flock = flock(Vec::new());
    flock.set_count(1000, FIELD, SimulationSettings::default().cruise_speed);
    for _ in 0..1200 {
        flock.step(
            1.0 / 60.0,
            FIELD,
            &mut MouseField::default(),
            &SimulationSettings::default(),
            &[],
        );
    }
    let average_speed = flock
        .birds
        .iter()
        .map(|bird| bird.velocity.length())
        .sum::<f32>()
        / 1000.0;
    assert!(
        average_speed > 65.0,
        "average flight speed fell to {average_speed:.1}"
    );
}

#[test]
fn natural_flight_forms_locally_aligned_groups() {
    let mut flock = flock(Vec::new());
    flock.set_count(300, FIELD, SimulationSettings::default().cruise_speed);
    for _ in 0..1200 {
        flock.step(
            1.0 / 60.0,
            FIELD,
            &mut MouseField::default(),
            &SimulationSettings::default(),
            &[],
        );
    }
    flock.grid.rebuild(&flock.birds, NEIGHBOR_RADIUS);
    let mut alignment = 0.0;
    let mut pairs = 0;
    for (index, bird) in flock.birds.iter().enumerate() {
        flock
            .grid
            .for_each_neighbor(&flock.birds, index, |neighbor, _, _| {
                alignment += bird
                    .velocity
                    .normalize_or_zero()
                    .dot(flock.birds[neighbor].velocity.normalize_or_zero());
                pairs += 1;
            });
    }
    assert!(pairs > 300, "birds should have formed groups");
    assert!(
        alignment / pairs as f32 > 0.5,
        "neighbor alignment was {:.2}",
        alignment / pairs as f32
    );
}

#[test]
fn mouse_repulsion_near_an_edge_turns_smoothly_without_a_velocity_flip() {
    let mut flock = flock(vec![Bird::new(Vec2::new(590.0, 0.0), Vec2::X * MAX_SPEED)]);
    let mut mouse = MouseField {
        cursor: Some(Vec2::new(580.0, 0.0)),
        repelling: true,
        ..default()
    };
    for _ in 0..600 {
        let previous_velocity = flock.birds[0].velocity;
        flock.step(
            1.0 / 60.0,
            FIELD,
            &mut mouse,
            &SimulationSettings::default(),
            &[],
        );
        assert!(flock.birds[0].position.x.abs() < FIELD.x);
        assert!(
            flock.birds[0].velocity.distance(previous_velocity) <= 850.0 / 60.0 + 0.01,
            "boundary must steer, not teleport or flip velocity"
        );
    }
}

#[test]
fn disconnected_flocks_reconnect_during_free_flight() {
    let mut flock = flock(
        (0..50)
            .map(|index| {
                let group = if index < 25 { -1.0 } else { 1.0 };
                let slot = index % 25;
                Bird::new(
                    Vec2::new(
                        group * 200.0 + (slot % 5) as f32 * 8.0,
                        (slot / 5) as f32 * 8.0,
                    ),
                    Vec2::X * group * 135.0,
                )
            })
            .collect(),
    );
    let mut reconnected = false;
    for _ in 0..1800 {
        flock.step(
            1.0 / 60.0,
            FIELD,
            &mut MouseField::default(),
            &SimulationSettings::default(),
            &[],
        );
        flock.grid.rebuild(&flock.birds, NEIGHBOR_RADIUS);
        let mut visited = [false; 50];
        let mut pending = vec![0];
        visited[0] = true;
        while let Some(index) = pending.pop() {
            flock
                .grid
                .for_each_neighbor(&flock.birds, index, |other, _, _| {
                    if !visited[other] {
                        visited[other] = true;
                        pending.push(other);
                    }
                });
        }
        if visited.iter().filter(|&&member| member).count() >= 40 {
            reconnected = true;
            break;
        }
    }
    assert!(
        reconnected,
        "the separated flocks must recover a shared neighborhood"
    );
}

#[test]
fn neighbor_radius_changes_which_birds_are_seen() {
    let mut flock = flock(vec![
        Bird::new(Vec2::ZERO, Vec2::X * 100.0),
        Bird::new(Vec2::new(90.0, 0.0), Vec2::Y * 100.0),
    ]);
    let settings = SimulationSettings {
        neighbor_radius: 120.0,
        regroup_weight: 0.0,
        ..default()
    };
    flock.grid.rebuild(&flock.birds, settings.neighbor_radius);
    assert!(
        flock
            .flight_forces(0, Vec2::ZERO, FIELD, &MouseField::default(), &settings, &[])
            .steering
            .y
            > 0.0
    );
}

#[test]
fn zero_separation_weight_removes_local_repulsion() {
    let flock = flock(vec![
        Bird::new(Vec2::new(-4.0, 0.0), Vec2::Y * 100.0),
        Bird::new(Vec2::new(4.0, 0.0), Vec2::Y * 100.0),
    ]);
    let settings = SimulationSettings {
        separation_weight: 0.0,
        alignment_weight: 0.0,
        cohesion_weight: 0.0,
        regroup_weight: 0.0,
        ..default()
    };
    assert_eq!(
        flock
            .flight_forces(0, Vec2::ZERO, FIELD, &MouseField::default(), &settings, &[])
            .steering
            .x,
        0.0
    );
}

#[test]
fn nearest_neighbors_ignore_distant_heading_majority() {
    let mut birds = vec![
        Bird::new(Vec2::ZERO, Vec2::Y * 100.0),
        Bird::new(Vec2::new(10.0, 0.0), Vec2::X * 100.0),
        Bird::new(Vec2::new(20.0, 0.0), Vec2::X * 100.0),
    ];
    birds.extend((0..8).map(|i| Bird::new(Vec2::new(0.0, 40.0 + i as f32), -Vec2::X * 100.0)));
    let flock = flock(birds);
    let settings = SimulationSettings {
        limit_neighbors: true,
        neighbor_limit: 2,
        separation_weight: 0.0,
        cohesion_weight: 0.0,
        regroup_weight: 0.0,
        ..default()
    };
    assert!(
        flock
            .flight_forces(0, Vec2::ZERO, FIELD, &MouseField::default(), &settings, &[])
            .steering
            .x
            > 0.0
    );
}

#[test]
fn flight_recovers_cruise_speed_after_a_turn() {
    let mut flock = flock(vec![Bird::new(Vec2::ZERO, Vec2::X * 10.0)]);
    let settings = SimulationSettings {
        cruise_speed: 160.0,
        individuality: 0.0,
        ..default()
    };
    for _ in 0..120 {
        flock.step(
            1.0 / 60.0,
            FIELD,
            &mut MouseField::default(),
            &settings,
            &[],
        );
    }
    assert!(flock.birds[0].velocity.length() > 150.0);
}

#[test]
fn fast_falcon_pass_leaves_a_decaying_escape_response() {
    let mut flock = flock(vec![Bird::new(Vec2::new(0.0, 20.0), Vec2::X * 100.0)]);
    let settings = SimulationSettings::default();
    let mut mouse = MouseField::default();
    mouse.update(Some(Vec2::new(-300.0, 0.0)), true, 1.0 / 60.0);
    mouse.update(Some(Vec2::new(300.0, 0.0)), true, 1.0 / 60.0);
    flock.step(1.0 / 60.0, FIELD, &mut mouse, &settings, &[]);
    assert!(flock.birds[0].alarm > 0.0);
    let alarm = flock.birds[0].alarm;
    mouse.update(Some(Vec2::new(400.0, 300.0)), true, 1.0 / 60.0);
    flock.step(1.0 / 60.0, FIELD, &mut mouse, &settings, &[]);
    assert!(flock.birds[0].alarm > 0.0 && flock.birds[0].alarm < alarm);
    mouse.update(Some(Vec2::new(400.0, 300.0)), false, 1.0 / 60.0);
    flock.step(1.0 / 60.0, FIELD, &mut mouse, &settings, &[]);
    assert_eq!(flock.birds[0].alarm, 0.0);
}

#[test]
fn slow_turns_at_high_cruise_speed_stay_in_a_small_field() {
    let field = Vec2::new(320.0, 210.0);
    let settings = SimulationSettings {
        cruise_speed: 240.0,
        turn_rate: 60.0,
        ..default()
    };
    let mut flock = flock(Vec::new());
    flock.set_count(100, field, SimulationSettings::default().cruise_speed);
    for _ in 0..1200 {
        flock.step(
            1.0 / 60.0,
            field,
            &mut MouseField::default(),
            &settings,
            &[],
        );
        for bird in &flock.birds {
            assert!(
                bird.position.x.abs() <= field.x && bird.position.y.abs() <= field.y,
                "bird left the field at {:?}",
                bird.position
            );
        }
    }
}

fn replay_falcon_motion(render_hz: usize) -> Vec<Vec2> {
    let settings = SimulationSettings {
        count: 40,
        ..default()
    };
    let mut flock = flock(Vec::new());
    flock.set_count(40, FIELD, 135.0);
    let mut mouse = MouseField::default();
    mouse.update(Some(Vec2::new(-240.0, 0.0)), true, 0.0);
    let mut accumulator = 0.0_f64;
    for frame in 1..=render_hz {
        let t = frame as f32 / render_hz as f32;
        mouse.update(
            Some(Vec2::new(-240.0 + 480.0 * t, 0.0)),
            true,
            1.0 / render_hz as f32,
        );
        accumulator += 1.0 / render_hz as f64;
        while accumulator + 1e-9 >= 1.0 / 60.0 {
            flock.step(1.0 / 60.0, FIELD, &mut mouse, &settings, &[]);
            accumulator -= 1.0 / 60.0;
        }
    }
    flock.birds.iter().map(|bird| bird.position).collect()
}

#[test]
fn thirty_fps_falcon_motion_matches_sixty_fps() {
    let expected = replay_falcon_motion(60);
    let actual = replay_falcon_motion(30);
    for (index, (a, b)) in actual.iter().zip(expected).enumerate() {
        assert!(a.distance(b) < 0.05, "bird {index}: {a:?} vs {b:?}");
    }
}

#[test]
fn high_fps_falcon_motion_matches_sixty_fps() {
    let expected = replay_falcon_motion(60);
    let actual = replay_falcon_motion(144);
    for (index, (a, b)) in actual.iter().zip(expected).enumerate() {
        assert!(a.distance(b) < 0.05, "bird {index}: {a:?} vs {b:?}");
    }
}

#[test]
fn nearest_distance_finds_birds_beyond_the_perception_radius() {
    let flock = flock(vec![
        Bird::new(Vec2::ZERO, Vec2::X * 100.0),
        Bird::new(Vec2::new(300.0, 0.0), Vec2::X * 100.0),
    ]);
    assert_eq!(flock.grid.nearest_distance(&flock.birds, 0), Some(300.0));
}

#[test]
fn inspection_shows_the_neighbors_used_for_alignment_and_cohesion() {
    let flock = flock(vec![
        Bird::new(Vec2::ZERO, Vec2::Y * 100.0),
        Bird::new(Vec2::new(10.0, 0.0), Vec2::X * 100.0),
        Bird::new(Vec2::new(-30.0, 0.0), -Vec2::X * 100.0),
    ]);
    let settings = SimulationSettings {
        limit_neighbors: true,
        neighbor_limit: 1,
        ..Default::default()
    };
    let details = flock.inspect(0, &MouseField::default(), &settings, &flock.grid);
    assert_eq!(details.neighbors, vec![1]);
    assert!(details.rules.alignment.x > 0.0);
    assert!(details.rules.separation.x < 0.0);
}

#[test]
fn a_bird_passes_a_circle_without_entering_it() {
    let obstacle = Obstacle {
        center: Vec2::ZERO,
        radius: 40.0,
    };
    let settings = SimulationSettings {
        cohesion_weight: 0.0,
        regroup_weight: 0.0,
        ..Default::default()
    };
    let mut flock = flock(vec![Bird::new(Vec2::new(-200.0, 0.0), Vec2::X * 135.0)]);
    let mut reached_far_side = false;
    for _ in 0..360 {
        flock.step(
            1.0 / 60.0,
            FIELD,
            &mut MouseField::default(),
            &settings,
            &[obstacle],
        );
        let position = flock.birds[0].position;
        assert!(
            position.length() >= 44.99,
            "bird entered obstacle: {position:?}"
        );
        reached_far_side |= position.x > 100.0;
    }
    assert!(reached_far_side, "bird failed to pass the obstacle");
}

#[test]
fn dense_attraction_cannot_push_birds_inside_obstacles() {
    let settings = SimulationSettings {
        count: 200,
        cruise_speed: 240.0,
        turn_rate: 60.0,
        attraction_strength: 300.0,
        ..Default::default()
    };
    let obstacles = [
        Obstacle {
            center: Vec2::ZERO,
            radius: 55.0,
        },
        Obstacle {
            center: Vec2::new(200.0, 100.0),
            radius: 45.0,
        },
    ];
    let mut flock = flock(Vec::new());
    flock.set_count(200, FIELD, 240.0);
    flock.exclude_obstacles(&obstacles, FIELD);
    let mut mouse = MouseField::default();
    mouse.update(Some(Vec2::ZERO), false, 0.0);
    for _ in 0..360 {
        flock.step(1.0 / 60.0, FIELD, &mut mouse, &settings, &obstacles);
        for (index, bird) in flock.birds.iter().enumerate() {
            for circle in obstacles {
                assert!(
                    bird.position.distance(circle.center) >= circle.radius + 4.99,
                    "bird {index} entered {circle:?} at {:?}",
                    bird.position
                );
            }
        }
    }
}

#[test]
fn tight_preset_forms_a_denser_flock_than_loose_preset() {
    use crate::{observation::Observation, presets::Preset};
    let mut tight_settings = SimulationSettings {
        count: 300,
        ..Default::default()
    };
    let mut loose_settings = tight_settings.clone();
    Preset::Tight.apply(&mut tight_settings);
    Preset::Loose.apply(&mut loose_settings);
    let mut tight = flock(Vec::new());
    let mut loose = flock(Vec::new());
    tight.set_count(300, FIELD, tight_settings.cruise_speed);
    loose.set_count(300, FIELD, loose_settings.cruise_speed);
    for _ in 0..600 {
        tight.step(
            1.0 / 60.0,
            FIELD,
            &mut MouseField::default(),
            &tight_settings,
            &[],
        );
        loose.step(
            1.0 / 60.0,
            FIELD,
            &mut MouseField::default(),
            &loose_settings,
            &[],
        );
    }
    let mut tight_metrics = Observation::default();
    let mut loose_metrics = Observation::default();
    tight_metrics.measure(&tight);
    loose_metrics.measure(&loose);
    let tight_spacing = tight_metrics.mean_nearest.unwrap();
    let loose_spacing = loose_metrics.mean_nearest.unwrap();
    println!("tight spacing={tight_spacing:.2}, loose spacing={loose_spacing:.2}");
    assert!(
        tight_spacing < loose_spacing * 0.9,
        "tight={tight_spacing}, loose={loose_spacing}"
    );
}

#[test]
fn shrinking_the_window_does_not_place_a_bird_inside_an_edge_obstacle() {
    let bounds = Vec2::new(320.0, 210.0);
    let obstacle = Obstacle {
        center: Vec2::new(300.0, 0.0),
        radius: 100.0,
    };
    let mut flock = flock(vec![Bird::new(Vec2::new(600.0, 0.0), Vec2::X * 135.0)]);
    flock.resize_field(bounds, &[obstacle]);
    assert!(
        flock.birds[0].position.distance(obstacle.center) >= 105.0,
        "{:?}",
        flock.birds[0].position
    );
    assert!(flock.birds[0].position.x.abs() <= bounds.x);
}
