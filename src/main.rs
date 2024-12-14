/* J O U R N E Y   O F   T H E
 *    M E A D O W   K I N G
 * -=-=-=-=-=-=-=-=-=-=-=-=-=-
 * a "clone" of Journey of the Prairie King
 * by SpielProfessor. Original JOPK by ConcernedApe
 * Assets: Kenney tiny dungeon 16x16 tiles
 * Licensed under the FreeBSD license
 * Copyright (c) SpielProfessor 2024
 *
 *
 * T O D O
 * Effects
 * Room transitions
 * More enemies
 * Improve enemy AI
 */
use std::collections::HashMap;
use macroquad::prelude::*;
use macroquad::prelude::scene::Node;
use macroquad::rand::gen_range;
use macroquad_tiled::{load_map, Map};
use macroquad_canvas_2d::Canvas2D;
use crate::player::{key_inputs, update_fixed, Player};
use macroUtils::{include_texture, wrapping, GameUpdate};
use macroUtils::timemanager::TimeManager;
use crate::bullet::{Bullet};
use crate::collision::CollisionType;
use crate::enemy::{initialize_enemy_textures, Enemy};
use crate::items::Item;

mod player;
mod items;
mod enemy;
mod bullet;
mod collision;

pub const GAME_SCREEN_MAIN: Rect = Rect { x: 94., y: 0., w: 256., h: 256. };
pub const SPEED: f32 = 1.;
pub const TILE_SIZE: f32 = 16.;
pub const STD_TIMER_MAX: i32 = 3000;

pub struct GameState {
    pub assets: HashMap<&'static str, Texture2D>,
    pub tilemap: Map,
    pub tilemap_old: Map,
    pub canvas: Canvas2D,
    pub player: Player,
    pub bullets: Vec<Bullet>,
    pub enemies: Vec<Enemy>,
    pub debug: bool,
    pub enemies_killcount: i32,
    pub kill_goal: Vec<i32>,
    pub current_stage: usize,
    pub tilemaps: Vec<&'static str>,
    pub stage_timer: i32,
}

impl GameState {
    async fn new() -> Self {
        let mut assets = HashMap::new();

        assets.insert("tiles", include_texture!("../assets/tilemap.png"));
        assets.insert("player", include_texture!("../assets/player.png"));
        assets.insert("item::coffee", include_texture!("../assets/coffee.png"));
        assets.insert("item::quickshoot", include_texture!("../assets/quickshoot.png"));
        assets.insert("next_effect", include_texture!("../assets/next_effect.png"));
        assets.insert("menu::death", include_texture!("../assets/death_text.png"));
        assets.insert("menu::title", include_texture!("../assets/title.png"));
        assets.insert("menu::paused", include_texture!("../assets/paused.png"));


        let mut tilemap = load_map(include_str!("../map.json"), &[("assets/tilemap.png", assets.get("tiles").unwrap().clone())], &[]).unwrap();
        let mut tilemap_old = load_map(include_str!("../map.json"), &[("assets/tilemap.png", assets.get("tiles").unwrap().clone())], &[]).unwrap();

        let mut canvas = Canvas2D::new(350., 256.);
        canvas.get_texture_mut().set_filter(FilterMode::Nearest);
        GameState {
            debug: false,
            assets,
            tilemap,
            canvas,
            player: Default::default(),
            bullets: vec![],
            enemies: vec![],
            enemies_killcount: 0,
            kill_goal: vec![200],
            current_stage: 0,
            tilemaps: vec![include_str!("../map.json"), include_str!("../map2.json")],
            stage_timer: STD_TIMER_MAX,
            tilemap_old,
        }
    }
}


const SHOOT_COOLDOWN_MAX: i32 = 30;

#[macroquad::main("Journey of the Meadow King")]
async fn main() {
    let mut current_state = 1;            // 0: playing, 1: main menu, 2: dead, 3: PAUSE menu, 4: DEBUG settings
    set_default_filter_mode(FilterMode::Nearest);
    // draw loading screen
    draw_text("Initializing...", 30., 50., 50., WHITE);
    next_frame().await;

    // initialize enemy texture atlas
    initialize_enemy_textures().await;
    // initialize game state
    let mut gs = GameState::new().await;

    // initialize font
    let mut font = load_ttf_font_from_bytes(include_bytes!("../assets/font.ttf")).unwrap();
    font.set_filter(FilterMode::Nearest);


    let mut time_handle = TimeManager::new();

    let mut main_menu_selected = 0;
    let mut main_menu_items = vec!["Start", "Settings", "Quit Game"];

    // we loop already here, because we want to be able to restart later (and we dont have a scene management system)
    'outer: loop {
        let mut shoot_cooldown = 0;
        let mut fixed_update_time: f32 = 0.;
        // initialize collision detection map
        let mut collision_map: Vec<CollisionType> = vec![];

        // loop over every collidable tile in the `main` layer and add to the collision map
        for (x, y, tile) in gs.tilemap.tiles("main", None) {
            collision_map.push(if tile.is_some() {
                CollisionType::Solid(x, y)
            } else {
                CollisionType::Empty(x, y)
            });
        }

        // enemy spawn areas
        let mut spawnpoints: Vec<Vec2> = vec![];
        for (x, y, tile) in gs.tilemap.tiles("spawnable", None) {
            if tile.is_some() {
                spawnpoints.push(vec2(x as f32 * TILE_SIZE + 1., y as f32 * TILE_SIZE + 1.));
            }
        }

        // if it is some(x), draw damage animation on enemy x
        let mut draw_damage_animation: Option<usize> = None;
        //
        // M A I N  L O O P
        //
        'game_loop: loop {
            if current_state == 1 {
                //////////////////////////////
                //
                // Main Menu
                //
                //////////////////////////////
                clear_background(BLACK);
                // width of the title: 540
                // height:              50


                // draw menu title
                if screen_width() > 3840. && screen_height() > 2160. {
                    draw_text_centred("Journey of the", &font, -500.);
                    draw_text_centred("Meadow King", &font, -200.);
                } else if screen_width() > 530. && screen_height() > 300. {
                    draw_texture_ex(gs.assets.get("menu::title").unwrap(), screen_width() / 2. - 270., 70., WHITE, DrawTextureParams {
                        dest_size: None,
                        ..Default::default()
                    });
                } else {
                    draw_text_centred("Journey of the", &font, -50.);
                    draw_text_centred("Meadow King", &font, -25.);
                }
                // draw menu options
                for i in 0..main_menu_items.len() {
                    if i == main_menu_selected {
                        draw_text_centred(format!("> {} <", main_menu_items.get(i).unwrap()).as_str(), &font, 50. * i as f32);
                    } else {
                        draw_text_centred(main_menu_items.get(i).unwrap(), &font, 50. * i as f32);
                    }
                }

                // get keyboard inputs
                if is_key_pressed(KeyCode::Enter) {
                    if main_menu_selected == 0 {
                        current_state = 0;
                    } else if main_menu_selected == 1 {
                        current_state = 4;
                    } else if main_menu_selected == 2 {
                        next_frame().await;
                        println!("[INFO] Exiting...");
                        break 'outer;
                    }
                }
                if is_key_pressed(KeyCode::Up) {
                    if main_menu_selected != 0 {
                        main_menu_selected -= 1;
                    } else {
                        main_menu_selected = main_menu_items.len() - 1;
                    }
                } else if is_key_pressed(KeyCode::Down) {
                    main_menu_selected += 1;
                    if main_menu_selected >= main_menu_items.len() {
                        main_menu_selected = 0
                    }
                }
            } else if current_state == 0 {

                //////////////////////////////
                //
                // G A M E  U P D A T E
                //
                //////////////////////////////
                // update

                time_handle.update(&mut draw_damage_animation);

                // debug key
                if is_key_pressed(KeyCode::F3) {
                    gs.debug = !gs.debug;
                }
                // switch to Pause menu
                if is_key_pressed(KeyCode::Escape) {
                    current_state = 3;
                }

                //
                // F I X E D  U P D A T E
                //
                fixed_update_time += get_frame_time();
                while fixed_update_time >= 1. / 60. {
                    // update room timer
                    if gs.stage_timer > 0 {
                        gs.stage_timer -= 1;
                        // spawn enemy if we're still gaming
                        if gen_range(0, 69) == 0 {
                            gs.enemies.push(Enemy::new_random(&spawnpoints));
                        }
                    }


                    // player update
                    update_fixed(&mut gs);
                    // player keys
                    key_inputs(&mut gs, &mut shoot_cooldown, &collision_map);

                    // decrease shoot cooldown
                    if shoot_cooldown > 0 {
                        shoot_cooldown -= 1;
                    }


                    // update bullets & enemies fixed
                    for bullet in &mut gs.bullets {
                        bullet.update();
                    }

                    for enemy in &mut gs.enemies {
                        enemy.update(&gs.player, &collision_map);
                        // check for player & enemy collision
                        if Rect::new(gs.player.coords.x, gs.player.coords.y, gs.player.wh.x, gs.player.wh.y)
                            .overlaps(&Rect::new(enemy.coords.x, enemy.coords.y, enemy.wh.x, enemy.wh.y))
                        {
                            gs.player.health -= 1;
                            gs.stage_timer += 50;
                            gs.enemies.clear();
                            gs.player.reset_coords();
                            break;
                        }
                    }
                    //
                    // D E A T H
                    //
                    if gs.player.health <= 0 {
                        current_state = 2;    // set to death mode
                        break 'game_loop;
                    }
                    // collision detection enemies/bullets
                    let mut enemy_index = 0;
                    for enemy in gs.enemies.clone() {
                        let mut bullet_index = 0;
                        for bullet in gs.bullets.clone() {
                            let enemy_hitbox = Rect::new(enemy.coords.x, enemy.coords.y, enemy.wh.x, enemy.wh.y);
                            if enemy_hitbox.contains(bullet.coords) {
                                gs.bullets.remove(bullet_index);
                                let mut current_enemy = gs.enemies.get_mut(enemy_index);
                                if current_enemy.is_some() {
                                    if current_enemy.expect("ERR: Expected enemy to be something but was nothing!").damage(gs.player.strength) {
                                        gs.enemies.remove(enemy_index);
                                        gs.enemies_killcount += 1;
                                    } else {
                                        draw_damage_animation = Some(enemy_index);
                                    }
                                } else {
                                    println!("WARN: Sorry! Couldn't kill enemy");
                                }
                            }
                            bullet_index += 1;
                        }
                        enemy_index += 1;
                    }

                    // check for next stage transition
                    if gs.stage_timer <= 0 && gs.player.coords.y.round() >= (GAME_SCREEN_MAIN.y + GAME_SCREEN_MAIN.h).round() - 16. {
                        // reset game state
                        gs.stage_timer = STD_TIMER_MAX;
                        gs.current_stage += 1;
                        gs.player.reset_coords();
                        gs.tilemap_old = gs.tilemap;
                        gs.tilemap = load_map(gs.tilemaps.get(gs.current_stage).unwrap(), &[("assets/tilemap.png", gs.assets.get("tiles").unwrap().clone())], &[]).unwrap();
                        let mut transition_timer = GAME_SCREEN_MAIN.h;
                        // reset collision, spawnpoints, ..
                        // initialize collision detection map
                        collision_map.clear();
                        // loop over every collidable tile in the `main` layer and add to the collision map
                        for (x, y, tile) in gs.tilemap.tiles("main", None) {
                            collision_map.push(if tile.is_some() {
                                CollisionType::Solid(x, y)
                            } else {
                                CollisionType::Empty(x, y)
                            });
                        }

                        // enemy spawn areas
                        spawnpoints.clear();
                        for (x, y, tile) in gs.tilemap.tiles("spawnable", None) {
                            if tile.is_some() {
                                spawnpoints.push(vec2(x as f32 * TILE_SIZE + 1., y as f32 * TILE_SIZE + 1.));
                            }
                        }


                        //
                        // draw transition
                        //
                        loop {
                            gs.canvas.set_camera();
                            clear_background(BLACK);
                            let mut old_tilemap = GAME_SCREEN_MAIN;
                            old_tilemap.y = transition_timer.round() - GAME_SCREEN_MAIN.h;      // offset view of old room
                            // draw old room
                            gs.tilemap_old.draw_tiles("background", old_tilemap, None);
                            gs.tilemap_old.draw_tiles("main", old_tilemap, None);
                            gs.tilemap_old.draw_tiles("objects", old_tilemap, None);

                            let mut new_tilemap = GAME_SCREEN_MAIN;
                            new_tilemap.y = transition_timer.round();                           // offset view of new room
                            // draw new room
                            gs.tilemap.draw_tiles("background", new_tilemap, None);
                            gs.tilemap.draw_tiles("main", new_tilemap, None);
                            gs.tilemap.draw_tiles("objects", new_tilemap, None);

                            // draw player
                            let mut player_y = gs.player.coords.y + GAME_SCREEN_MAIN.y + GAME_SCREEN_MAIN.h / 2. - 10.;
                            if transition_timer <= GAME_SCREEN_MAIN.h / 2. {
                                player_y = GAME_SCREEN_MAIN.y + GAME_SCREEN_MAIN.h / 2. + transition_timer.round() - 8.;
                            }
                            draw_texture_ex(
                                gs.assets.get("player").unwrap(),
                                gs.player.coords.x + GAME_SCREEN_MAIN.x,
                                player_y,
                                WHITE,
                                DrawTextureParams {
                                    flip_x: gs.player.flipped,
                                    ..Default::default()
                                },
                            );


                            // increase offset
                            transition_timer -= 0.5;


                            gs.canvas.draw_to_screen();
                            next_frame().await;
                            if transition_timer <= 0. {
                                break;
                            }
                        }
                        //
                        // T R A N S I T I O N  E N D
                        //
                    }


                    fixed_update_time -= 1. / 60.;
                }
                //
                // F I X E D  E N D
                //

                // draw

                gs.canvas.set_camera();
                clear_background(BLACK);
                // draw tiles
                gs.tilemap.draw_tiles("background", GAME_SCREEN_MAIN, None);
                gs.tilemap.draw_tiles("main", GAME_SCREEN_MAIN, None);
                gs.tilemap.draw_tiles("objects", GAME_SCREEN_MAIN, None);

                // draw enemies
                let mut index = 0;
                for enemy in &gs.enemies {
                    if let Some(x) = draw_damage_animation {
                        if x == index {
                            enemy.draw(true);
                            time_handle.delay(0.1, |draw_damage_animation| { *draw_damage_animation = None; });
                        } else {
                            enemy.draw(false);
                        }
                    } else {
                        enemy.draw(false);
                    }
                    index += 1;
                }

                // draw bullets
                for bullet in &gs.bullets {
                    bullet.draw();
                }

                // draw player
                draw_texture_ex(
                    gs.assets.get("player").unwrap(),
                    gs.player.coords.x + GAME_SCREEN_MAIN.x,
                    gs.player.coords.y + GAME_SCREEN_MAIN.y,
                    WHITE,
                    DrawTextureParams {
                        flip_x: gs.player.flipped,
                        ..Default::default()
                    },
                );

                // draw debug hitboxes
                if gs.debug {
                    // draw player hitbox
                    draw_circle(gs.player.coords.x + GAME_SCREEN_MAIN.x, gs.player.coords.y + GAME_SCREEN_MAIN.y, 2., GREEN);
                    //draw_circle(gs.player.coords.x + gs.player.wh.x + GAME_SCREEN_MAIN.x, gs.player.coords.y + gs.player.wh.y + GAME_SCREEN_MAIN.y, 1., RED);
                    draw_rectangle_lines(gs.player.coords.x + GAME_SCREEN_MAIN.x, gs.player.coords.y + GAME_SCREEN_MAIN.y, gs.player.wh.x, gs.player.wh.y, 1., GREEN);
                    draw_rectangle(gs.player.coords.x + GAME_SCREEN_MAIN.x, gs.player.coords.y + gs.player.wh.y + GAME_SCREEN_MAIN.y, gs.player.wh.x, 1., RED);

                    // draw map hitbox

                    for tile in &collision_map {
                        if let CollisionType::Solid(x, y) = tile {
                            draw_rectangle_lines(*x as f32 * TILE_SIZE + GAME_SCREEN_MAIN.x, *y as f32 * TILE_SIZE + GAME_SCREEN_MAIN.y, TILE_SIZE, TILE_SIZE, 2., RED);
                        }
                    }

                    // enemy hitboxes
                    for enemy in &gs.enemies {
                        draw_rectangle_lines(GAME_SCREEN_MAIN.x + enemy.coords.x, enemy.coords.y, enemy.wh.x, enemy.wh.y, 1., BLUE);
                    }
                }

                // HUD
                // effect duration
                if gs.player.effect_duration > 0 {
                    draw_rectangle(10., 19. - (gs.player.effect_duration / 64) as f32 / 2., 5., (gs.player.effect_duration / 64) as f32, if (gs.player.effect_duration / 64) / 2 <= 1 { RED } else { GREEN });
                }
                // time to play the stage for
                if gs.stage_timer > 0 {
                    draw_rectangle(10., 80. - (gs.stage_timer / 64) as f32 / 2., 5., (gs.stage_timer / 64) as f32, if (gs.stage_timer / 64) / 2 <= 1 { RED } else { GREEN });
                }

                // next effect background
                draw_texture_ex(
                    &gs.assets.get("next_effect").unwrap(),
                    20.,
                    10.,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(20., 20.)), ..Default::default() },
                );

                // next effect
                if gs.player.held_effect == Some(Item::Speed) {
                    draw_texture(&gs.assets.get("item::coffee").unwrap(), 22., 12., WHITE);
                } else if gs.player.held_effect == Some(Item::Quickshoot) {
                    draw_texture(&gs.assets.get("item::quickshoot").unwrap(), 22., 12., WHITE);
                }
                // player HP
                for i in 0..gs.player.health {
                    draw_rectangle(i as f32 * 6., 25., 5., 5., DARKGREEN);
                }


                // draw canvas to screen, continue without scaled screen
                gs.canvas.draw_to_screen();


                // debug HUD
                if gs.debug {
                    draw_text(format!("FPS: {}", get_fps()).as_str(), 10., 10., 20., WHITE);
                    draw_text(format!("Player Coords: {}/{}", gs.player.coords.x as i32, gs.player.coords.y as i32).as_str(), 10., 25., 20., WHITE);
                    draw_text(format!("-> Tile: {}/{}", (gs.player.coords.x / TILE_SIZE) as i32, (gs.player.coords.y / TILE_SIZE) as i32).as_str(), 10., 40., 20., WHITE);
                }
            } else if current_state == 2 {

                /////////////////////////////////
                //
                // D E A T H   S T A T E
                //
                ////////////////////////////////
                clear_background(BLACK);
                gs.canvas.set_camera();
                draw_texture(gs.assets.get("menu::death").unwrap(), GAME_SCREEN_MAIN.x + 15. + 2., (GAME_SCREEN_MAIN.y + GAME_SCREEN_MAIN.h) / 2. - /*27: Font size*/48. / 2. + 2., BLACK);
                draw_texture(gs.assets.get("menu::death").unwrap(), GAME_SCREEN_MAIN.x + 15., (GAME_SCREEN_MAIN.y + GAME_SCREEN_MAIN.h) / 2. - /*27: Font size*/48. / 2., WHITE);
                if is_key_pressed(KeyCode::Space) {
                    gs.canvas.draw_to_screen();
                    next_frame().await;
                    gs = GameState::new().await;
                    current_state = 1;
                    break 'game_loop;
                }

                gs.canvas.draw_to_screen();
            } else if current_state == 3 {
                ////////////////////////////////
                //
                // P A U S E   M E N U
                //
                ////////////////////////////////
                clear_background(BLACK);
                gs.canvas.set_camera();
                draw_rectangle(0., 0., GAME_SCREEN_MAIN.x, gs.canvas.height(), BLACK);
                draw_texture(gs.assets.get("menu::paused").unwrap(), 2., 25., WHITE);

                if is_key_pressed(KeyCode::Escape) {
                    current_state = 0;
                } else if is_key_pressed(KeyCode::Q) {
                    next_frame().await;
                    break 'outer;
                }

                gs.canvas.draw_to_screen();
            } else if current_state == 4 {
                ////////////////////////////////
                //
                // S E T T I N G S
                //
                ////////////////////////////////
                clear_background(BLACK);
                draw_text_centred("Not yet implemented!", &font, 0.);
                draw_text_centred("Press [SPACE] to go to the main menu", &font, 50.);
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    current_state = 1;
                }
            }


            // wait for next frame
            next_frame().await;
        }
    }
}


/// draw text using specified font in the centre with the offset `offset`
fn draw_text_centred(text: &str, font: &Font, offset: f32) {
    let start_size = measure_text(text, Some(font), (screen_height() / 20.) as u16, 1.);
    draw_text_ex(text, screen_width() / 2. - start_size.width / 2., screen_height() / 2. - start_size.height / 2. + offset, TextParams {
        font: Some(&font),
        font_size: (screen_height() / 20.) as u16,
        font_scale: 1.0,
        font_scale_aspect: 1.0,
        rotation: 0.0,
        color: WHITE,
    });
}
fn draw_text_centred_ex(text: &str, font: &Font, offset: f32, width: f32, height: f32) {
    let start_size = measure_text(text, Some(font), (height / 20.) as u16, 1.);
    draw_text_ex(text, width / 2. - start_size.width / 2., height / 2. - start_size.height / 2. + offset, TextParams {
        font: Some(&font),
        font_size: (height / 20.) as u16,
        font_scale: 1.0,
        font_scale_aspect: 1.0,
        rotation: 0.0,
        color: WHITE,
    });
}